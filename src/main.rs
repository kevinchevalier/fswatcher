use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::env;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let watch_path = args[1].clone();
    
    println!("Starting file system watcher for: {}", watch_path);
    
    if !Path::new(&watch_path).exists() {
        eprintln!("Warning: Directory {} does not exist!", watch_path);
        return Ok(());
    }
    
    // Create a channel for raw events
    let (raw_tx, raw_rx): (mpsc::Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) = mpsc::channel();
    
    // Create a raw watcher first
    let mut raw_watcher = notify::recommended_watcher(move |res| {
        if let Err(e) = raw_tx.send(res) {
            println!("* Failed to send raw event: {:?}", e);
        }
    })?;
    
    // Start watching with the raw watcher
    raw_watcher.watch(Path::new(&watch_path), RecursiveMode::Recursive)?;
    
    // Create a debounced file watcher with 100ms debounce
    let mut debouncer: Debouncer<RecommendedWatcher, FileIdMap> = new_debouncer(
        Duration::from_millis(100),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    println!("\n🎯 * DEBOUNCED EVENTS:");
                    for event in events {
                        println!("  Debounced event: {:?}", event.event.kind);
                        println!("  Paths: {:?}", event.event.paths);
                        println!("  Time: {:?}", event.time);
                    }
                    println!("{}", "=".repeat(50));
                }
                Err(errors) => {
                    for error in errors {
                        println!("* Debounced watch error: {:?}", error);
                    }
                }
            }
        },
    )?;
    
    // Start watching the directory recursively with debouncer too
    debouncer.watch(Path::new(&watch_path), RecursiveMode::Recursive)?;
    
    println!("File watcher started successfully. Watching for changes in: {}", watch_path);
    println!("Press Ctrl+C to stop...");
    println!("{}", "=".repeat(60));
    
    // Keep the program running and process raw events
    loop {
        // Check for raw events
        match raw_rx.try_recv() {
            Ok(event) => {
                match event {
                    Ok(event) => {
                        println!("\n⚡ * RAW EVENT:");
                        println!("  Raw event kind: {:?}", event.kind);
                        println!("  Raw paths: {:?}", event.paths);
                        if let Some(tracker) = &event.attrs.tracker() {
                            println!("  Tracker: {:?}", tracker);
                        }
                        println!("{}", "-".repeat(30));
                    }
                    Err(e) => {
                        println!("* Raw watch error: {:?}", e);
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                println!("Raw event channel disconnected");
                return Ok(());
            }
        }
        
        sleep(Duration::from_millis(10)).await;
    }
}
