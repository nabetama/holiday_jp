use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;
use chrono::NaiveDate;

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;
    use httptest::{matchers::request, responders::status_code, Expectation, Server};
    use std::{fs::remove_file, io::prelude::*};

    #[test]
    fn test_generate() -> Result<()> {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/dummy.csv")).respond_with(
                status_code(200).body("dummy-title,dummy-title2\r\n2022/01/01,HOLIDAY!\r\n"),
            ),
        );

        let url = server.url("/dummy.csv");
        generate(&url.to_string(), "dummy.rs")?;

        let mut f = File::open("dummy.rs").expect("file not found");

        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        assert!(contents.contains("HOLIDAY!"));

        remove_file("dummy.rs")?;

        Ok(())
    }
}

#[tokio::main]
pub async fn generate(url: &str, out_file: &str) -> Result<()> {
    let body = reqwest::get(url)
        .await?
        .text_with_charset("shift-jis")
        .await?;

    let mut rdr = csv::Reader::from_reader(body.as_bytes());

    let header = "use std::collections::HashMap;
    pub fn dates() -> HashMap<String, String> {
        let res = HashMap::from([    
    ";

    let footer = "    ]);
    res
}";

    let path = Path::new(&out_file);
    let mut writer = BufWriter::new(File::create(&path)?);

    write!(&mut writer, "{}", header)?;
    for record in rdr.records() {
        let record = record?;

        let dt = NaiveDate::parse_from_str(&String::from(&record[0]), "%Y/%m/%d")?;
        writeln!(
            &mut writer,
            "(\"{}\".to_string(), \"{}\".to_string()),",
            dt.format("%Y/%m/%d"),
            &record[1]
        )?;
    }
    write!(&mut writer, "{}", footer)?;
    writer.flush()?;

    Ok(())
}
