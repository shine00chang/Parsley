use super::*;

pub fn option<'a, T, P> (p: P) -> impl Parser<'a, Option<T>>
where 
    P: Parser<'a, T>
{
    move |buf| {
        if let Ok((buf, o)) = p.parse(buf) {
            Ok((buf, Some(o)))
        } else {
            Ok((buf, None))
        }
    }
}

pub fn and<'a, A, B, PA, PB> (a: PA, b: PB) -> impl Parser<'a, (A, B)> 
where
    PA: Parser<'a, A>,
    PB: Parser<'a, B>,
{
    move |buf| {
        a.parse(buf)
            .and_then(|(buf, res_a)| 
                b.parse(buf)
                    .map(|(buf, res_b)| (buf, (res_a, res_b) ))
            )
    }
}

pub fn or<'a, P1, P2, A>(parser1: P1, parser2: P2) -> impl Parser<'a, A>
where
    P1: Parser<'a, A>,
    P2: Parser<'a, A>,
{
    move |input| match parser1.parse(input) {
        ok @ Ok(_) => ok,
        Err(_) => parser2.parse(input),
    }
} 


pub fn zero_or_more<'a, A, P> (p: P) -> impl Parser<'a, Vec<A>> 
where
    P: Parser<'a, A>
{
    move |buf| {
        let mut v = vec![];
        let mut buf_out = buf;
        while let Ok((buf, out)) = p.parse(buf_out) {
            v.push(out);

            buf_out = buf;
        }
        Ok((buf_out, v))
    }
}

pub fn one_or_more<'a, A, P> (p: P) -> impl Parser<'a, Vec<A>> 
where
    P: Parser<'a, A>
{
    move |buf| {
        let mut v = vec![];
        let mut buf_out = buf;
        while let Ok((buf, out)) = p.parse(buf_out) {
            v.push(out);

            buf_out = buf;
        }
        if v.is_empty() {
            par_err(buf, "none of pattern found in 'one_or_more'")
        } else {
            Ok((buf_out, v))
        }
    }
}

pub fn prefix<'a, T, P> (s: &'a str, p: P) -> impl Parser<'a, T> 
where
    P: Parser<'a, T>
{
    map( 
        and(
            parse_literal(s),
            p
        ),
        |(_, a)| a
    )
}

pub fn suffix<'a, T, P> (s: &'a str, p: P) -> impl Parser<'a, T> 
where
    P: Parser<'a, T>
{
    map( 
        and(
            p,
            parse_literal(s) 
        ),
        |(a, _)| a
    )
}

pub fn surround<'a, T, P> (a: &'a str, b: &'a str, p: P) -> impl Parser<'a, T> 
where
    P: Parser<'a, T>
{
    prefix(
        a, 
        suffix(b, p) 
    )
}

pub fn parse_literal<'a> (lit: &'a str) -> impl Parser<'a, &str> {
    move |buf: &'a str| match buf.get(0..lit.len()) {
        Some(s) if s == lit => Ok((&buf[lit.len()..], lit)),
        _ => par_err_s(buf, format!("Literal '{}' not found", lit))
    } 
}


pub fn parse_literals<'a> (lits: Vec<&'a str>) -> impl Parser<'a, &str> {
    move |buf: &'a str| {
        for lit in lits.iter() {
            match buf.get(0..lit.len()) {
                Some(s) if &s == lit => return Ok((&buf[lit.len()..], &buf[0..lit.len()])),
                _ => continue
            }
        }
        par_err_s(buf, format!("Literal '{:?}' not found", lits))
    }
}

pub fn parse_tok_with_rule<'a, R> (rule: R) -> impl Parser<'a, String> 
where
    R: Fn (char) -> bool
{
    move |buf: &'a str| {
        let mut tok = String::new();
        let mut iter = buf.chars();

        match iter.next() {
            Some(c) if rule(c) => tok.push(c),
            _ => return par_err(buf, "First character does not satisfy rule")
        }
        while let Some(c) = iter.next() {
            if rule(c) {
                tok.push(c);
            } else { break }
        }
        if tok.is_empty() {
            par_err(buf, "Empty Token.")
        } else {
            Ok((&buf[tok.len()..], tok))
        }
    }
}


pub fn parse_number<'a> () -> impl Parser<'a, f64> {
    move |buf: &'a str| {
        let num_rule = |c: char| {
            c.is_ascii_digit() || c == '.'
        };
        let (buf, tok) = parse_tok_with_rule(num_rule).parse(buf)?;
        if let Ok(num) = tok.parse::<f64>() {
            Ok((buf, num))
        } else {
            par_err(buf, "could not parse into number")
        }
    }
}

pub fn parse_identifier<'a> () -> impl Parser<'a, String> {
    move |input: &'a str| {
        let rule = |c: char| {
            c.is_alphanumeric() || c == '_'
        };
        let (buf, tok) = parse_tok_with_rule(rule).parse(input)?;

        if tok.chars().next().unwrap().is_ascii_digit() { return par_err(buf, "identifier cannot start with digit") }
        if KEYWORDS.contains(&tok.as_str())             { return par_err(buf, "found keyword, cannot be used as identifier") }

        Ok((buf, tok))
    }
}

pub fn map<'a, A, B, P, F> (parser: P, functor: F) -> impl Parser<'a, B> 
where 
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |buf: &'a str| -> ParseRes<'a, B> {
        parser.parse(buf)
            .map(|(b, out): (&str, A)| (b, functor(out)))
    }
}

const KEYWORDS: [&str; 4] = [
    "true",
    "false",
    "if",
    "eval"
];
