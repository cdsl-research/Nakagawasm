use async_recursion::async_recursion;
use futures::future::join_all;

const HOST: &str = "http://0.0.0.0:1234";

fn extract_links(body: &str) -> Vec<String> {
    let dom = tl::parse(body, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    // if let Some(anchor) =
    let mut a_hrefs = dom
        .query_selector("a[href]")
        .unwrap()
        .map(|node| {
            node.get(parser)
                .unwrap()
                .as_tag()
                .unwrap()
                .attributes()
                .get("href")
                .flatten()
                .unwrap()
                .as_utf8_str()
                .into_owned()
        })
        .collect::<Vec<_>>();
    let mut img_srcs = dom
        .query_selector("img[src]")
        .unwrap()
        .map(|node| {
            node.get(parser)
                .unwrap()
                .as_tag()
                .unwrap()
                .attributes()
                .get("src")
                .flatten()
                .unwrap()
                .as_utf8_str()
                .into_owned()
        })
        .collect::<Vec<_>>();

    a_hrefs.append(&mut img_srcs);
    a_hrefs
}

#[async_recursion]
async fn clawle(url: &str) -> anyhow::Result<()> {
    let resp = reqwest::get(url).await?;

    tracing::info!("{}: {}", resp.status(), url);

    if let Ok(text) = resp.text().await {
        let urls = extract_links(&text)
            .iter()
            .map(|link| format!("{}/{}", HOST, link))
            .collect::<Vec<_>>();

        let contents = urls.iter().map(|link| clawle(&link.as_str()));

        join_all(contents)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_ansi(false).init();

    clawle(HOST).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let links = extract_links(r#"hello <img src="/foo-bar.png"> <a href="/567">good</a>"#);
        assert_eq!(
            links,
            ["/567", "/foo-bar.png"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );

        let links = extract_links(r#"<img><a>not attrs!</a>"#);
        assert!(links.is_empty());

        let links = extract_links(r#"no tags!"#);
        assert!(links.is_empty());
    }
}
