use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadSummary {
    pub dataset: String,
    pub corpus_archive: String,
    pub queries_archive: String,
    pub qrels_test: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterializationSummary {
    pub dataset: String,
    pub documents: usize,
    pub test_queries: usize,
    pub output_dir: String,
}

pub fn download_scifact_dataset(
    base_url: &str,
    qrels_base_url: &str,
    out_dir: &Path,
) -> Result<DownloadSummary> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create download dir {}", out_dir.display()))?;
    fs::create_dir_all(out_dir.join("qrels"))
        .with_context(|| format!("create qrels dir {}", out_dir.join("qrels").display()))?;

    download_asset(
        &join_url(base_url, "corpus.jsonl.gz"),
        &out_dir.join("corpus.jsonl.gz"),
    )?;
    download_asset(
        &join_url(base_url, "queries.jsonl.gz"),
        &out_dir.join("queries.jsonl.gz"),
    )?;
    download_asset(
        &join_url(qrels_base_url, "test.tsv"),
        &out_dir.join("qrels").join("test.tsv"),
    )?;

    Ok(DownloadSummary {
        dataset: "scifact".to_string(),
        corpus_archive: "corpus.jsonl.gz".to_string(),
        queries_archive: "queries.jsonl.gz".to_string(),
        qrels_test: "qrels/test.tsv".to_string(),
    })
}

pub fn materialize_scifact_dir(
    source_dir: &Path,
    out_dir: &Path,
) -> Result<MaterializationSummary> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create materialized dir {}", out_dir.display()))?;
    fs::create_dir_all(out_dir.join("qrels")).with_context(|| {
        format!(
            "create materialized qrels dir {}",
            out_dir.join("qrels").display()
        )
    })?;

    let qrels_path = source_dir.join("qrels").join("test.tsv");
    let qrel_rows = read_qrels(&qrels_path)?;
    let wanted_query_ids: HashSet<_> = qrel_rows.iter().map(|row| row.query_id.clone()).collect();

    let corpus_records = read_jsonl_gz::<CorpusRecord>(&source_dir.join("corpus.jsonl.gz"))?;
    let queries = read_jsonl_gz::<QueryRecord>(&source_dir.join("queries.jsonl.gz"))?;

    let mut documents = 0;
    for record in corpus_records {
        let filename = out_dir.join(format!("{}.txt", sanitize_doc_id(&record.id)));
        let mut body = String::new();
        if let Some(title) = record
            .title
            .as_deref()
            .filter(|title| !title.trim().is_empty())
        {
            body.push_str(title.trim());
            body.push_str("\n\n");
        }
        body.push_str(record.text.trim());

        fs::write(&filename, body)
            .with_context(|| format!("write materialized document {}", filename.display()))?;
        documents += 1;
    }

    let mut query_file = fs::File::create(out_dir.join("test-queries.tsv")).with_context(|| {
        format!(
            "create query file {}",
            out_dir.join("test-queries.tsv").display()
        )
    })?;
    writeln!(query_file, "query-id\ttext").context("write query header")?;

    let mut test_queries = 0;
    for query in queries {
        if wanted_query_ids.contains(&query.id) {
            writeln!(
                query_file,
                "{}\t{}",
                query.id,
                query.text.replace('\n', " ")
            )
            .context("write materialized query row")?;
            test_queries += 1;
        }
    }

    fs::copy(&qrels_path, out_dir.join("qrels").join("test.tsv")).with_context(|| {
        format!(
            "copy qrels {} -> {}",
            qrels_path.display(),
            out_dir.join("qrels").join("test.tsv").display()
        )
    })?;

    Ok(MaterializationSummary {
        dataset: "scifact".to_string(),
        documents,
        test_queries,
        output_dir: out_dir.display().to_string(),
    })
}

#[derive(Debug, Deserialize)]
struct CorpusRecord {
    #[serde(rename = "_id")]
    id: String,
    #[serde(default)]
    title: Option<String>,
    text: String,
}

#[derive(Debug, Deserialize)]
struct QueryRecord {
    #[serde(rename = "_id")]
    id: String,
    text: String,
}

#[derive(Debug)]
struct QrelRow {
    query_id: String,
}

fn download_asset(url: &str, path: &Path) -> Result<()> {
    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("download {}", url))?;
    let bytes = response
        .body_mut()
        .read_to_vec()
        .with_context(|| format!("read response body from {}", url))?;

    fs::write(path, bytes).with_context(|| format!("write asset {}", path.display()))?;
    Ok(())
}

fn join_url(base: &str, file: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), file)
}

fn read_jsonl_gz<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let file = fs::File::open(path).with_context(|| format!("open archive {}", path.display()))?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut rows = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("read line from {}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let row = serde_json::from_str(&line)
            .with_context(|| format!("parse jsonl row from {}", path.display()))?;
        rows.push(row);
    }

    Ok(rows)
}

fn read_qrels(path: &Path) -> Result<Vec<QrelRow>> {
    let file = fs::File::open(path).with_context(|| format!("open qrels {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read qrels line from {}", path.display()))?;
        if index == 0 || line.trim().is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let query_id = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing query-id in {}", path.display()))?;
        let _corpus_id = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing corpus-id in {}", path.display()))?;
        let _score = parts
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("missing score in {}", path.display()))?;

        rows.push(QrelRow {
            query_id: query_id.to_string(),
        });
    }

    Ok(rows)
}

fn sanitize_doc_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '\0' => '_',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tempfile::tempdir;

    use super::{download_scifact_dataset, materialize_scifact_dir};

    #[test]
    fn download_scifact_dataset_fetches_expected_assets() {
        let fixture = build_fixture_files();
        let corpus_listener = TcpListener::bind("127.0.0.1:0").expect("bind corpus server");
        let qrels_listener = TcpListener::bind("127.0.0.1:0").expect("bind qrels server");
        let corpus_addr = corpus_listener.local_addr().expect("corpus addr");
        let qrels_addr = qrels_listener.local_addr().expect("qrels addr");

        let corpus_thread = serve_fixture(
            corpus_listener,
            fixture.corpus_gz.clone(),
            fixture.queries_gz.clone(),
        );
        let qrels_thread = serve_qrels(qrels_listener, fixture.test_tsv.clone());

        let out_dir = tempdir().expect("download dir");
        let summary = download_scifact_dataset(
            &format!("http://{}", corpus_addr),
            &format!("http://{}", qrels_addr),
            out_dir.path(),
        )
        .expect("download summary");

        assert_eq!(summary.dataset, "scifact");
        assert!(out_dir.path().join("corpus.jsonl.gz").exists());
        assert!(out_dir.path().join("queries.jsonl.gz").exists());
        assert!(out_dir.path().join("qrels/test.tsv").exists());

        corpus_thread.join().expect("corpus server join");
        qrels_thread.join().expect("qrels server join");
    }

    #[test]
    fn materialize_scifact_dir_writes_docs_queries_and_qrels() {
        let source_dir = tempdir().expect("source dir");
        let fixture = build_fixture_files();

        fs::write(source_dir.path().join("corpus.jsonl.gz"), fixture.corpus_gz)
            .expect("write corpus");
        fs::write(
            source_dir.path().join("queries.jsonl.gz"),
            fixture.queries_gz,
        )
        .expect("write queries");
        fs::create_dir_all(source_dir.path().join("qrels")).expect("create qrels dir");
        fs::write(source_dir.path().join("qrels/test.tsv"), fixture.test_tsv).expect("write qrels");

        let out_dir = tempdir().expect("materialized dir");
        let summary = materialize_scifact_dir(source_dir.path(), out_dir.path())
            .expect("materialize summary");

        assert_eq!(summary.documents, 2);
        assert!(out_dir.path().join("doc-a.txt").exists());
        assert!(out_dir.path().join("doc-b.txt").exists());
        assert!(out_dir.path().join("test-queries.tsv").exists());
        assert!(out_dir.path().join("qrels/test.tsv").exists());
    }

    struct FixtureFiles {
        corpus_gz: Vec<u8>,
        queries_gz: Vec<u8>,
        test_tsv: Vec<u8>,
    }

    fn build_fixture_files() -> FixtureFiles {
        let corpus = [
            r#"{"_id":"doc-a","title":"Alpha","text":"rust search benchmark corpus","metadata":{}}"#,
            r#"{"_id":"doc-b","title":"Beta","text":"semantic rerank later story","metadata":{}}"#,
        ]
        .join("\n");
        let queries = [
            r#"{"_id":"q-1","text":"rust benchmark"}"#,
            r#"{"_id":"q-2","text":"rerank story"}"#,
        ]
        .join("\n");
        let qrels = "query-id\tcorpus-id\tscore\nq-1\tdoc-a\t1\nq-2\tdoc-b\t1\n";

        FixtureFiles {
            corpus_gz: gzip_bytes(corpus.as_bytes()),
            queries_gz: gzip_bytes(queries.as_bytes()),
            test_tsv: qrels.as_bytes().to_vec(),
        }
    }

    fn gzip_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).expect("write gzip");
        encoder.finish().expect("finish gzip")
    }

    fn serve_fixture(
        listener: TcpListener,
        corpus_gz: Vec<u8>,
        queries_gz: Vec<u8>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept fixture");
                let mut request = [0_u8; 1024];
                let read = stream.read(&mut request).expect("read request");
                let line = String::from_utf8_lossy(&request[..read]);
                let body = if line.starts_with("GET /corpus.jsonl.gz ") {
                    &corpus_gz
                } else {
                    &queries_gz
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(response.as_bytes()).expect("write header");
                stream.write_all(body).expect("write body");
            }
        })
    }

    fn serve_qrels(listener: TcpListener, test_tsv: Vec<u8>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept qrels");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).expect("read qrels request");

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                test_tsv.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write qrels header");
            stream.write_all(&test_tsv).expect("write qrels body");
        })
    }
}
