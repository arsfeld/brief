mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::transcribe::RecordingState::default())
        .manage(commands::transcribe::WhisperState::default())
        .invoke_handler(tauri::generate_handler![
            commands::notes::list_notes,
            commands::notes::read_note,
            commands::notes::write_note,
            commands::notes::delete_note,
            commands::ai::enhance_note,
            commands::transcribe::start_recording,
            commands::transcribe::stop_recording,
            commands::transcribe::stop_and_transcribe,
            commands::transcribe::check_whisper_model,
            commands::transcribe::download_whisper_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
