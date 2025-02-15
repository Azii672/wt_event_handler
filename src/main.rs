use std::thread::sleep;
use std::{time, fs};
use std::path::Path;

use log::*;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use rand::Rng;
use chrono::offset::Local;

use crate::recent_name_to_index::convert;
use crate::scrapers::forum_news::html_processor_wt_forums;
use crate::scrapers::wt_changelog::html_processor_wt_changelog;
use crate::scrapers::wt_news::html_processor_wt_news;
use crate::webhook_handler::{handle_simple_webhook, handle_wt_news_webhook};
use chrono::{Datelike, Timelike};

mod webhook_handler;
mod recent_name_to_index;
mod scrapers;
mod json_to_structs;


#[tokio::main]
async fn main() {
	if Path::new("log/latest.log").exists() {
		let time = format!("{}_{}_{}_{}-{}-{}", Local::now().year(), Local::now().month(), Local::now().day(), Local::now().hour(), Local::now().minute(), Local::now().second());
		fs::rename("log/latest.log", format!("log/old/{}.log", time)).expect("Could not rename latest log file");
	}

	let logfile = FileAppender::builder()
		.encoder(Box::new(PatternEncoder::new("{l} {d(%Y-%m-%d %H:%M:%S)} {l} - {m}\n")))
		.build("log/latest.log").unwrap();

	let config = Config::builder()
		.appender(Appender::builder().build("logfile", Box::new(logfile)))
		.build(Root::builder()
			.appender("logfile")
			.build(LevelFilter::Info)).unwrap();

	log4rs::init_config(config).unwrap();

	println!("Started client");
	info!("Started client");

	let news_index = convert("warthunder_news");
	let changelog_index = convert("warthunder_changelog");
	let forum_index = convert("forums");

	loop {
		let wt_news_content = html_processor_wt_news(news_index).await;
		if wt_news_content != "fetch_failed" {
			handle_wt_news_webhook(wt_news_content, news_index).await;
		};

		let wt_changelog = html_processor_wt_changelog(changelog_index).await;
		if wt_changelog != "fetch_failed" {
			handle_simple_webhook(wt_changelog, changelog_index).await;
		};

		let forum_news = html_processor_wt_forums(forum_index).await;
		if forum_news != "fetch_failed" {
			handle_simple_webhook(forum_news, forum_index).await;
		};

		// Cool down to prevent rate limiting and excessive performance impact
		let wait = rand::thread_rng().gen_range(50..70);
		println!("Waiting for {} seconds", wait);
		info!("Waiting for {} seconds", wait);
		sleep(time::Duration::from_secs(wait))
	}
}