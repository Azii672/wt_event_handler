use std::fs;

use serenity::http::Http;
use serenity::model::prelude::Webhook;

pub async fn handle_webhook(content: String, index: usize) {

	// let embed = Embed::fake(|mut e| {
	//     // e.title("Cool news and that shit");
	//     // e.description("Very nice");
	//     e.url(content);
	//     e
	// });

	#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
	pub struct Root {
		pub targets: Vec<Target>,
	}

	#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
	pub struct Target {
		pub name: String,
		pub recent_url: String,
		pub domain: String,
	}

	let cache_raw = fs::read_to_string("recent.json").expect("Cannot read file");
	let mut cache: Root = serde_json::from_str(&cache_raw).expect("Json cannot be read");

	if !content.contains("No match found") {
		if cache.targets[index].recent_url != content {
			println!("New post found, hooking now");
			execute_webhooks(&content).await;
			cache.targets[index].recent_url = content;
			let write = serde_json::to_string(&cache).unwrap();
			fs::write("recent.json", write).expect("Couldn't write to file");
		} else {
			println!("Content was recently fetched and is not new");
		}
	} else {
		println!("Content was either not a match")
	}

	async fn execute_webhooks(content: &String) {
		#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
		pub struct WebhookAuth {
			pub hooks: Vec<Hooks>,
		}

		#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
		pub struct Hooks {
			pub name: String,
			pub token: String,
			pub uid: u64,
		}

		let token_raw = fs::read_to_string("assets/discord_token.json").expect("Cannot read file");
		let webhook_auth: WebhookAuth = serde_json::from_str(&token_raw).expect("Json cannot be read");

		for hook in webhook_auth.hooks {

			let uid = hook.uid;
			let token = hook.token;

			let my_http_client = Http::new_with_token(&token);

			let webhook = match my_http_client.get_webhook_with_token(uid, &token).await {
				Err(why) => {
					println!("{}", why);
					panic!("")
				}
				Ok(hook) => hook,
			};
			webhook.execute(my_http_client, false, |w| {
				w.content(&format!("[{a}]({a})", a = content));
				w.username("The WT news bot");
				// w.embeds(vec![embed]);
				w
			})
				.await
				.unwrap();
		}
	}
}