use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EnhanceRequest {
    pub content: String,
    pub mode: String, // "polish" | "summarize" | "action_items" | "decisions"
    pub provider: String, // "local" | "openai" | "anthropic"
    pub api_key: Option<String>,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EnhanceResponse {
    pub result: String,
}

/// Routes AI requests to the appropriate provider.
/// Local: calls llama-server sidecar on localhost:8080
/// Cloud: calls provider API directly with user's key
#[tauri::command]
pub async fn enhance_note(request: EnhanceRequest) -> Result<EnhanceResponse, String> {
    let prompt = build_prompt(&request.content, &request.mode);

    match request.provider.as_str() {
        "local" => call_local_llama(&prompt, request.model.as_deref()).await,
        "openai" => {
            let key = request.api_key.ok_or("OpenAI API key required")?;
            call_openai(&prompt, &key, request.model.as_deref()).await
        }
        "anthropic" => {
            let key = request.api_key.ok_or("Anthropic API key required")?;
            call_anthropic(&prompt, &key, request.model.as_deref()).await
        }
        _ => Err(format!("Unknown provider: {}", request.provider)),
    }
}

fn build_prompt(content: &str, mode: &str) -> String {
    match mode {
        "polish" => format!(
            "You are a meeting notes editor. Polish the following raw notes: fix grammar, add structure with headers, keep the author's voice. Output only the improved notes in Markdown.\n\n{content}"
        ),
        "summarize" => format!(
            "Summarize the following meeting notes in 3-5 concise bullet points. Output only the bullets in Markdown.\n\n{content}"
        ),
        "action_items" => format!(
            "Extract all action items from the following meeting notes. For each, note the owner if mentioned and any deadline. Output as a Markdown checklist.\n\n{content}"
        ),
        "decisions" => format!(
            "Extract all decisions made in the following meeting notes. Output as a Markdown list.\n\n{content}"
        ),
        _ => format!("Process the following meeting notes:\n\n{content}"),
    }
}

async fn call_local_llama(prompt: &str, _model: Option<&str>) -> Result<EnhanceResponse, String> {
    let client = reqwest::Client::new();

    #[derive(Serialize)]
    struct LlamaRequest {
        prompt: String,
        n_predict: i32,
        stream: bool,
    }

    #[derive(Deserialize)]
    struct LlamaResponse {
        content: String,
    }

    let body = LlamaRequest {
        prompt: prompt.to_string(),
        n_predict: 2048,
        stream: false,
    };

    let resp = client
        .post("http://localhost:8080/completion")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Local AI not available: {e}"))?;

    let data: LlamaResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(EnhanceResponse { result: data.content })
}

async fn call_openai(prompt: &str, api_key: &str, model: Option<&str>) -> Result<EnhanceResponse, String> {
    let client = reqwest::Client::new();
    let model = model.unwrap_or("gpt-4o-mini");

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }
    #[derive(Serialize)]
    struct OpenAIRequest {
        model: String,
        messages: Vec<Message>,
    }
    #[derive(Deserialize)]
    struct Choice {
        message: OpenAIMessage,
    }
    #[derive(Deserialize)]
    struct OpenAIMessage {
        content: String,
    }
    #[derive(Deserialize)]
    struct OpenAIResponse {
        choices: Vec<Choice>,
    }

    let body = OpenAIRequest {
        model: model.to_string(),
        messages: vec![Message { role: "user".to_string(), content: prompt.to_string() }],
    };

    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: OpenAIResponse = resp.json().await.map_err(|e| e.to_string())?;
    let result = data.choices.into_iter().next()
        .map(|c| c.message.content)
        .ok_or("Empty response from OpenAI")?;

    Ok(EnhanceResponse { result })
}

async fn call_anthropic(prompt: &str, api_key: &str, model: Option<&str>) -> Result<EnhanceResponse, String> {
    let client = reqwest::Client::new();
    let model = model.unwrap_or("claude-haiku-4-5-20251001");

    #[derive(Serialize)]
    struct AnthropicMessage {
        role: String,
        content: String,
    }
    #[derive(Serialize)]
    struct AnthropicRequest {
        model: String,
        max_tokens: i32,
        messages: Vec<AnthropicMessage>,
    }
    #[derive(Deserialize)]
    struct ContentBlock {
        text: String,
    }
    #[derive(Deserialize)]
    struct AnthropicResponse {
        content: Vec<ContentBlock>,
    }

    let body = AnthropicRequest {
        model: model.to_string(),
        max_tokens: 2048,
        messages: vec![AnthropicMessage { role: "user".to_string(), content: prompt.to_string() }],
    };

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: AnthropicResponse = resp.json().await.map_err(|e| e.to_string())?;
    let result = data.content.into_iter().next()
        .map(|b| b.text)
        .ok_or("Empty response from Anthropic")?;

    Ok(EnhanceResponse { result })
}
