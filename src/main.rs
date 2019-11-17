#[macro_use]
extern crate clap;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rusoto_cloudwatch;
extern crate rusoto_core;

use clap::{App, Arg, SubCommand};
use rusoto_cloudwatch::{
    CloudWatch, CloudWatchClient, ListMetricsError, ListMetricsInput, ListMetricsOutput, Metric,
};
use rusoto_core::{Region, RusotoError};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::env;
use std::path::Path;
use std::str::FromStr;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("region")
                .takes_value(true)
                .required(true)
                .long("region")
                .help("AWS region"),
        )
        .get_matches();

    let region = Region::from_str(matches.value_of("region").unwrap());
    let client = CloudWatchClient::new(region.unwrap());

    let lreq: ListMetricsInput = Default::default();
    list_metrics(client, lreq);
}

fn list_metrics(
    client: CloudWatchClient,
    req: ListMetricsInput,
) -> Option<RusotoError<ListMetricsError>> {
    match client.list_metrics(req).sync() {
        Ok(out) => {
            if let Some(mets) = out.metrics {
                for m in mets {
                    let mut dims_text: String = String::new();
                    if let Some(dims) = m.dimensions {
                        for (i, dim) in dims.iter().enumerate() {
                            dims_text.push_str(
                                format!(
                                    "{}{}:{}",
                                    if i == 0 { "" } else { "," },
                                    dim.name,
                                    dim.value
                                )
                                .as_str(),
                            );
                        }
                    }
                    println!(
                        "{},{},{}",
                        m.namespace.unwrap_or_default(),
                        m.metric_name.unwrap_or_default(),
                        dims_text,
                    );
                }
            }

            println!("next={:?}", out.next_token);

            if let Some(next) = out.next_token {
                let mut next_req: ListMetricsInput = Default::default();
                next_req.next_token = Some(next);
                return list_metrics(client, next_req);
            }

            return None;
        }
        Err(err) => {
            return Some(err);
        }
    };
}
