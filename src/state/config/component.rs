use std::sync::Arc;

use crate::state::{
    State,
    config::{Filter, Source},
};

#[derive(Debug, Clone)]
pub enum Component {
    Text(Arc<str>),
    Interpolation {
        source: Source,
        filters: Vec<Filter>,
    },
}
impl Component {
    async fn eval(&self, state: &State) -> anyhow::Result<Arc<str>> {
        let text = match self {
            Self::Text(text) => text.clone(),
            Self::Interpolation { source, filters } => {
                let text = source.eval(state).await?;
                if filters.is_empty() {
                    return Ok(text);
                }
                let mut text = text.to_string();

                for filter in filters {
                    text = filter.eval(state, &text);
                }

                text.into()
            }
        };
        Ok(text)
    }
}

pub async fn eval_status(state: &State) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    for part in state
        .config
        .status
        .iter()
        .map(|component| component.eval(state))
    {
        parts.push(part.await?);
    }
    Ok(parts.join(""))
}
