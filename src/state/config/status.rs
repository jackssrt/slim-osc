use std::sync::Arc;

use chumsky::{
    IterParser, Parser,
    error::Rich,
    extra,
    primitive::{choice, just, none_of},
    text,
};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
#[deny(dead_code)] // use it in the parser
pub enum Source {
    Separator,
    DateTime {
        format: String,
    },

    // Usage
    GpuUsage,
    CpuUsage,
    MemoryUsage,

    // Model
    GpuModel,
    CpuModel,

    Music {
        metadata: Arc<str>,
    },

    Command {
        // this is a string because its more efficient to use it later, can't convert a smart pointer to str to an &OsStr grr
        command: String,
    },
    Text(Arc<str>),
}

#[derive(Debug, Clone)]
#[deny(dead_code)] // use it in the parser
pub enum Filter {
    Uppercase,
    Lowercase,
    Trim,
    Subscript,
    Superscript,
    Marquee { length: usize, period: f64 },
    Truncate { length: usize },
}

#[derive(Debug, Clone)]
pub enum Component {
    Text(Arc<str>),
    Interpolation {
        source: Source,
        filters: Vec<Filter>,
    },
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Component>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    let one_fn_param = none_of(")")
        .repeated()
        .collect::<String>()
        .delimited_by(just("("), just(")"));

    // source\(depends on source\)+
    let source = choice((
        choice((
            just::<_, _, extra::Err<Rich<char>>>("sep"),
            just("separator"),
        ))
        .to(Source::Separator {}),
        choice((just("time"), just("date"), just("datetime")))
            .ignore_then(one_fn_param)
            .map(|format| Source::DateTime { format }),
        just("time").to(Source::DateTime {
            format: "%H:%M".into(),
        }),
        just("date").to(Source::DateTime {
            format: "%Y-%m-%d".into(),
        }),
        just("gpu_usage").to(Source::GpuUsage),
        just("cpu_usage").to(Source::CpuUsage),
        just("memory_usage").to(Source::MemoryUsage),
        just("gpu_model").to(Source::GpuModel),
        just("cpu_model").to(Source::CpuModel),
        just("command")
            .ignore_then(one_fn_param)
            .map(|command| Source::Command { command }),
        just("music")
            .ignore_then(one_fn_param)
            .map(|metadata| Source::Music {
                metadata: metadata.into(),
            }),
        just("text")
            .ignore_then(one_fn_param)
            .map(|text| Source::Text(text.into())),
    ));
    let filter = choice((
        choice((just("uppercase"), just("upper"))).to(Filter::Uppercase),
        choice((just("lowercase"), just("lower"))).to(Filter::Lowercase),
        choice((just("trim"), just("strip"))).to(Filter::Trim),
        choice((just("subscript"), just("sub"))).to(Filter::Subscript),
        choice((just("superscript"), just("super"))).to(Filter::Superscript),
        choice((just("marquee"), just("scroll"))).ignore_then(
            text::int(10)
                .to_slice()
                .then_ignore(just(",").padded())
                .then(text::int(10).to_slice().or_not())
                .delimited_by(just("("), just(")"))
                .map(|(left, right): (&str, Option<&str>)| Filter::Marquee {
                    // TODO: should probably not fail silently hereee...
                    length: left.parse().unwrap_or(10),
                    period: right.and_then(|x| x.parse().ok()).unwrap_or(10f64),
                }),
        ),
        choice((just("truncate"), just("trunc"), just("trun")))
            .ignore_then(text::int(10).to_slice().delimited_by(just("("), just(")")))
            .map(|length: &str| Filter::Truncate {
                length: length.parse().unwrap_or(10),
            }),
    ));
    // { source (| filter)* }
    let interpolation = source
        .then(
            (just("|").padded().ignore_then(filter).padded())
                .repeated()
                .collect(),
        )
        .map(|(source, filters)| Component::Interpolation { source, filters })
        .padded()
        .delimited_by(just("{"), just("}"));
    // any other string
    let text = none_of("{")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|s| Component::Text(s.into()));
    let component = choice((interpolation, text));
    let parser = component.repeated().collect::<Vec<_>>();
    // TODO: maybe recover from an error?
    let result = parser.parse(string.as_str()).into_result();
    tracing::debug!("parsed status: {:?}", result);

    result.map_err(|err| serde::de::Error::custom(format!("failed to parse component: {err:?}")))
}
