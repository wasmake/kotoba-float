# OpenAI subscription authentication review

Reviewed **2026-09-17** against official OpenAI documentation.

## ChatGPT subscription result: blocked

No documented integration lets this independent native desktop application obtain audio-transcription and text-generation inference through a user's ChatGPT subscription.

- [Sign in with ChatGPT](https://help.openai.com/en/articles/20001410-sign-in-with-chatgpt) is available to selected participating partners and provides identity information. The documentation explicitly says it does not grant an external application access to ChatGPT account data beyond identity and separately presented permissions. There is no public native-app registration, authorization endpoint, scope set, or entitlement for this project to implement.
- [OpenAI authentication for Codex](https://developers.openai.com/codex/auth) supports subscription sign-in on named Codex surfaces. It is not a general OAuth credential for standard API endpoints, audio transcription, or an unrelated desktop app.
- [Managing billing for ChatGPT and the API platform](https://help.openai.com/en/articles/9039756-managing-billing-for-chatgpt-and-the-api-platform) says the products have separate billing systems and API use is billed separately.
- The documented [GPT Action OAuth flow](https://developers.openai.com/api/docs/actions/authentication) is the reverse relationship: ChatGPT authenticates to a service supplied by the developer. It does not authenticate this desktop app to OpenAI inference.

### Independently checked capabilities

| Capability | Result | Missing capability |
|---|---|---|
| ChatGPT identity | Unsupported for this unregistered app | Public app registration and native callback specification |
| Subscription audio transcription | Unsupported | Subscription-scoped audio inference surface and entitlement |
| Subscription text translation | Unsupported | Subscription-scoped text inference surface and entitlement |
| Models and limits | Unavailable | No applicable integration, model list, or quota API |

Authentication success would not imply either inference capability. The candidates `gpt-4o-mini-transcribe` and `gpt-5-nano` are intentionally not used. API availability is not subscription availability.

## Optional API-key mode

The application now also supports an explicitly opt-in OpenAI Platform API key. This is **not** ChatGPT subscription access: requests are billed and limited through the user's OpenAI Platform account. The UI presents this distinction before connection.

Keys are stored only in Windows Credential Manager or macOS Keychain through `src-tauri/src/auth.rs`; they are never written to settings, renderer storage, or logs. Connecting and testing verify audio transcription and text translation independently. Disconnect clears the credential and cancels active processing. There is no shared key or automatic fallback.
