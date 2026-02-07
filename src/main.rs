use std::io::{self, Write};
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("waifu Finder for kitty terminal 🌸\n");
    
    print!("mode - [1] SFW  [2] NSFW (18+): ");
    io::stdout().flush()?;
    
    let mut mode_input = String::new();
    io::stdin().read_line(&mut mode_input)?;
    let mode = mode_input.trim();
    
    let (urls, mode_type) = if mode == "2" {
        println!("\n⚠️  NSFW MODE - You must be 18+ to continue.");
        print!("Confirm you are 18 or older (yes/no): ");
        io::stdout().flush()?;
        
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        
        if !confirm.trim().to_lowercase().starts_with('y') {
            println!("exiting");
            return Ok(());
        }
        
        println!("\n🔞 nsfw mode\n");
        (vec![
            "https://api.waifu.pics/nsfw/waifu",
            "https://api.waifu.pics/nsfw/neko",
            "https://api.waifu.pics/nsfw/trap",
        ], "nsfw")
    } else {
        println!("\n✅ sfw mode\n");
        (vec![
            "https://api.waifu.pics/sfw/waifu",
            "https://api.waifu.pics/sfw/neko",
            "https://api.waifu.pics/sfw/shinobu",
        ], "sfw")
    };
    
    println!("📥 pre-fetching images for instant loading...\n");
    
    // pre-fetch all images at once (async parallel loading)
    let mut tasks = vec![];
    for api_url in urls.iter() {
        let url = api_url.to_string();
        tasks.push(tokio::spawn(async move {
            fetch_image(&url).await
        }));
    }
    
    // wait for all images to download
    let mut images = vec![];
    for task in tasks {
        match task.await? {
            Ok(img) => images.push(img),
            Err(e) => eprintln!("❌ error fetching image: {}", e),
        }
    }
    
    println!("✅ all images loaded\n");
    
    // create downloads directory if it doesn't exist
    fs::create_dir_all("waifu_downloads")?;
    
    // display images one by one
    for (i, (image_data, image_url)) in images.iter().enumerate() {
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║ image {} of {}                                      ║", i + 1, images.len());
        println!("╚══════════════════════════════════════════════════════╝\n");
        
        display_image_kitty(image_data)?;
        
        println!("\n✨ URL: {}\n", image_url);
        
        print!("download this image? (y/n): ");
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if response.trim().to_lowercase().starts_with('y') {
            let filename = format!("waifu_downloads/waifu_{}_{}_{}.png", 
                mode_type,
                i + 1, 
                chrono::Utc::now().timestamp()
            );
            fs::write(&filename, image_data)?;
            println!("💾 saved to: {}\n", filename);
        }
        
        if i < images.len() - 1 {
            println!("press enter for next image...");
            let mut _input = String::new();
            io::stdin().read_line(&mut _input)?;
            println!("\n");
        }
    }
    
    println!("\nthanks for using Waifu Finder!");
    println!("downloads saved in: ./waifu_downloads/");
    
    Ok(())
}

async fn fetch_image(api_url: &str) -> Result<(Vec<u8>, String)> {
    // fetch the response
    let response = reqwest::get(api_url).await?;
    let json: serde_json::Value = response.json().await?;
    
    let image_url = json["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse image URL"))?
        .to_string();
    
    // download the image
    let image_response = reqwest::get(&image_url).await?;
    let image_data = image_response.bytes().await?.to_vec();
    
    Ok((image_data, image_url))
}

fn display_image_kitty(image_data: &[u8]) -> Result<()> {
    let encoded = general_purpose::STANDARD.encode(image_data);
    
    // kitty
    // split into chunks of 4096 bytes
    let chunk_size = 4096;
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(chunk_size)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();
    
    for (i, chunk) in chunks.iter().enumerate() {
        let m = if i == chunks.len() - 1 { 0 } else { 1 };
        
        if i == 0 {
            // first chunk with action and format
            print!("\x1b_Ga=T,f=100,m={};{}\x1b\\", m, chunk);
        } else {
            // subsequent chunks
            print!("\x1b_Gm={};{}\x1b\\", m, chunk);
        }
        io::stdout().flush()?;
    }
    
    println!(); // aesthetics
    
    Ok(())
}
