#[cfg(test)]
mod tests {
    use super::super::session::{ChatConfig, ChatMessage, ChatSession, MessageRole, PermissionMode, ToolUse};
    use super::super::storage;
    use std::time::SystemTime;

    // ====================================================================
    // ChatSession Tests
    // ====================================================================

    #[test]
    fn test_chat_session_new() {
        let project_path = "/test/project".to_string();
        let title = "Test Session".to_string();

        let session = ChatSession::new(project_path.clone(), title.clone());

        // Verify basic properties
        assert_eq!(session.project_path, project_path);
        assert_eq!(session.title, title);
        assert_eq!(session.message_count, 0);

        // Verify ID is UUID-like (36 characters with dashes)
        assert_eq!(session.id.len(), 36);
        assert!(session.id.contains('-'));

        // Verify timestamps are reasonable (within last minute)
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(session.created_at <= now);
        assert!(session.updated_at <= now);
        assert!(now - session.created_at < 60);

        // Verify created_at and updated_at are equal on creation
        assert_eq!(session.created_at, session.updated_at);
    }

    #[test]
    fn test_chat_session_new_creates_unique_ids() {
        let session1 = ChatSession::new("/test/project1".to_string(), "Chat 1".to_string());
        let session2 = ChatSession::new("/test/project2".to_string(), "Chat 2".to_string());

        assert_ne!(session1.id, session2.id);
    }

    // ====================================================================
    // ChatMessage Tests
    // ====================================================================

    #[test]
    fn test_chat_message_new() {
        let session_id = "test-session-123".to_string();
        let role = MessageRole::User;
        let content = "Hello, Claude!".to_string();

        let message = ChatMessage::new(session_id.clone(), role.clone(), content.clone());

        assert_eq!(message.session_id, session_id);
        assert_eq!(message.role, role);
        assert_eq!(message.content, content);
        assert_eq!(message.id.len(), 36); // UUID format
        assert!(message.id.contains('-'));
        assert!(message.tool_use.is_none());
        assert!(message.metadata.is_none());

        // Verify timestamp is reasonable
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(message.timestamp <= now);
        assert!(now - message.timestamp < 60);
    }

    #[test]
    fn test_chat_message_new_creates_unique_ids() {
        let session_id = "test-session".to_string();
        let msg1 = ChatMessage::new(session_id.clone(), MessageRole::User, "Message 1".to_string());
        let msg2 = ChatMessage::new(session_id.clone(), MessageRole::User, "Message 2".to_string());

        assert_ne!(msg1.id, msg2.id);
    }

    #[test]
    fn test_chat_message_different_roles() {
        let session_id = "test-session".to_string();

        let user_msg = ChatMessage::new(
            session_id.clone(),
            MessageRole::User,
            "User message".to_string(),
        );
        let assistant_msg = ChatMessage::new(
            session_id.clone(),
            MessageRole::Assistant,
            "Assistant message".to_string(),
        );
        let system_msg = ChatMessage::new(
            session_id.clone(),
            MessageRole::System,
            "System message".to_string(),
        );
        let tool_msg = ChatMessage::new(
            session_id.clone(),
            MessageRole::Tool,
            "Tool message".to_string(),
        );

        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(system_msg.role, MessageRole::System);
        assert_eq!(tool_msg.role, MessageRole::Tool);
    }

    // ====================================================================
    // ChatConfig Tests
    // ====================================================================

    #[test]
    fn test_chat_config_default() {
        let config = ChatConfig::default();

        assert_eq!(config.model, "sonnet");
        assert_eq!(config.permission_mode, PermissionMode::Default);
        assert!(config.max_tokens.is_none());
        assert!(config.temperature.is_none());
    }

    #[test]
    fn test_chat_config_serialization() {
        let config = ChatConfig {
            model: "opus".to_string(),
            permission_mode: PermissionMode::AcceptEdits,
            max_tokens: Some(2000),
            temperature: Some(0.7),
        };

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: ChatConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.model, "opus");
        assert_eq!(deserialized.permission_mode, PermissionMode::AcceptEdits);
        assert_eq!(deserialized.max_tokens, Some(2000));
        assert_eq!(deserialized.temperature, Some(0.7));
    }

    #[test]
    fn test_permission_mode_variants() {
        // Test all permission modes can be created and compared
        let default_mode = PermissionMode::Default;
        let accept_edits = PermissionMode::AcceptEdits;
        let bypass = PermissionMode::BypassPermissions;
        let plan = PermissionMode::Plan;

        assert_ne!(default_mode, accept_edits);
        assert_ne!(accept_edits, bypass);
        assert_ne!(bypass, plan);
    }

    // ====================================================================
    // ToolUse Tests
    // ====================================================================

    #[test]
    fn test_tool_use_serialization() {
        let tool_use = ToolUse {
            tool_name: "search".to_string(),
            input: serde_json::json!({"query": "test"}),
            output: Some("result".to_string()),
        };

        let json = serde_json::to_string(&tool_use).expect("Failed to serialize");
        let deserialized: ToolUse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.tool_name, "search");
        assert_eq!(deserialized.output, Some("result".to_string()));
    }

    #[test]
    fn test_message_with_tool_use() {
        let mut message = ChatMessage::new(
            "session-123".to_string(),
            MessageRole::Assistant,
            "I'll search for that".to_string(),
        );

        message.tool_use = Some(ToolUse {
            tool_name: "web_search".to_string(),
            input: serde_json::json!({"query": "test query"}),
            output: None,
        });

        let json = serde_json::to_string(&message).expect("Failed to serialize");
        let deserialized: ChatMessage =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert!(deserialized.tool_use.is_some());
        let tool = deserialized.tool_use.unwrap();
        assert_eq!(tool.tool_name, "web_search");
    }

    // ====================================================================
    // Storage Tests - Session Operations
    // ====================================================================

    #[test]
    fn test_storage_save_and_load_session() {
        let session = ChatSession::new("/test/project".to_string(), "Test Session".to_string());
        let messages = vec![ChatMessage::new(
            session.id.clone(),
            MessageRole::User,
            "Hello".to_string(),
        )];

        // Save session
        let save_result = storage::save_session(&session, &messages);
        assert!(save_result.is_ok(), "Failed to save session");

        // Load session
        let load_result = storage::load_session(&session.id);
        assert!(load_result.is_ok(), "Failed to load session");

        let (loaded_session, loaded_messages) = load_result.unwrap();
        assert_eq!(loaded_session.id, session.id);
        assert_eq!(loaded_session.project_path, session.project_path);
        assert_eq!(loaded_session.title, session.title);
        assert_eq!(loaded_messages.len(), 1);
        assert_eq!(loaded_messages[0].content, "Hello");
    }

    #[test]
    fn test_storage_save_session_creates_file() {
        let session = ChatSession::new("/test/project".to_string(), "Test Session".to_string());
        let messages = vec![];

        let result = storage::save_session(&session, &messages);
        assert!(result.is_ok(), "Failed to save session: {:?}", result);
    }

    #[test]
    fn test_storage_load_nonexistent_session() {
        // Use valid UUID format that doesn't exist
        let nonexistent_uuid = "00000000-0000-0000-0000-000000000000";
        let result = storage::load_session(nonexistent_uuid);
        assert!(result.is_err(), "Should fail to load nonexistent session");
        assert!(result
            .unwrap_err()
            .contains("Session not found"));
    }

    #[test]
    fn test_storage_delete_session() {
        let session = ChatSession::new("/test/project".to_string(), "Test Session".to_string());
        let messages = vec![];

        // Save session
        storage::save_session(&session, &messages).expect("Failed to save session");

        // Verify it was saved
        let load_result = storage::load_session(&session.id);
        assert!(load_result.is_ok(), "Session should exist before deletion");

        // Delete session
        let delete_result = storage::delete_session(&session.id);
        assert!(delete_result.is_ok(), "Failed to delete session");

        // Verify it was deleted
        let load_result = storage::load_session(&session.id);
        assert!(load_result.is_err(), "Session should not exist after deletion");
    }

    #[test]
    fn test_storage_list_sessions_filters_by_project() {
        let project1 = "/test/project1";
        let project2 = "/test/project2";

        let session1 = ChatSession::new(project1.to_string(), "Session 1".to_string());
        let session2 = ChatSession::new(project1.to_string(), "Session 2".to_string());
        let session3 = ChatSession::new(project2.to_string(), "Session 3".to_string());

        // Save all sessions
        storage::save_session(&session1, &[]).expect("Failed to save session1");
        storage::save_session(&session2, &[]).expect("Failed to save session2");
        storage::save_session(&session3, &[]).expect("Failed to save session3");

        // List sessions for project1
        let result = storage::list_sessions(project1);
        assert!(result.is_ok(), "Failed to list sessions");

        let sessions = result.unwrap();
        assert_eq!(sessions.len(), 2, "Should have 2 sessions for project1");
        assert!(sessions.iter().any(|s| s.id == session1.id));
        assert!(sessions.iter().any(|s| s.id == session2.id));

        // Cleanup
        let _ = storage::delete_session(&session1.id);
        let _ = storage::delete_session(&session2.id);
        let _ = storage::delete_session(&session3.id);
    }

    #[test]
    fn test_storage_list_sessions_sorts_by_updated_at() {
        let project = "/test/project";
        let mut session1 = ChatSession::new(project.to_string(), "Session 1".to_string());
        let mut session2 = ChatSession::new(project.to_string(), "Session 2".to_string());

        // Modify updated_at to control sort order
        session1.updated_at = 100;
        session2.updated_at = 200;

        storage::save_session(&session1, &[]).expect("Failed to save session1");
        storage::save_session(&session2, &[]).expect("Failed to save session2");

        let result = storage::list_sessions(project);
        assert!(result.is_ok());

        let sessions = result.unwrap();
        if sessions.len() >= 2 {
            // More recent (higher updated_at) should come first
            let session2_idx = sessions.iter().position(|s| s.id == session2.id).unwrap_or(0);
            let session1_idx = sessions.iter().position(|s| s.id == session1.id).unwrap_or(1);
            assert!(session2_idx < session1_idx, "Sessions should be sorted by updated_at descending");
        }

        // Cleanup
        let _ = storage::delete_session(&session1.id);
        let _ = storage::delete_session(&session2.id);
    }

    #[test]
    fn test_storage_list_sessions_empty_project() {
        let result = storage::list_sessions("/nonexistent/project/path");
        assert!(result.is_ok(), "Should return ok for nonexistent project");

        let sessions = result.unwrap();
        assert_eq!(sessions.len(), 0, "Should have no sessions for nonexistent project");
    }

    #[test]
    fn test_storage_update_session_metadata() {
        let mut session = ChatSession::new("/test/project".to_string(), "Original Title".to_string());
        let message = ChatMessage::new(
            session.id.clone(),
            MessageRole::User,
            "Hello".to_string(),
        );

        // Save session
        storage::save_session(&session, &[message]).expect("Failed to save session");

        // Update metadata
        session.title = "Updated Title".to_string();
        let update_result = storage::update_session_metadata(&session);
        assert!(update_result.is_ok(), "Failed to update session metadata");

        // Load and verify
        let (loaded_session, _) = storage::load_session(&session.id).expect("Failed to load session");
        assert_eq!(loaded_session.title, "Updated Title");

        // Cleanup
        let _ = storage::delete_session(&session.id);
    }

    #[test]
    fn test_storage_save_multiple_messages() {
        let session = ChatSession::new("/test/project".to_string(), "Test Session".to_string());
        let messages = vec![
            ChatMessage::new(
                session.id.clone(),
                MessageRole::User,
                "First message".to_string(),
            ),
            ChatMessage::new(
                session.id.clone(),
                MessageRole::Assistant,
                "Second message".to_string(),
            ),
            ChatMessage::new(
                session.id.clone(),
                MessageRole::User,
                "Third message".to_string(),
            ),
        ];

        storage::save_session(&session, &messages).expect("Failed to save session");

        let (_, loaded_messages) = storage::load_session(&session.id).expect("Failed to load");
        assert_eq!(loaded_messages.len(), 3);
        assert_eq!(loaded_messages[0].content, "First message");
        assert_eq!(loaded_messages[1].content, "Second message");
        assert_eq!(loaded_messages[2].content, "Third message");

        // Cleanup
        let _ = storage::delete_session(&session.id);
    }

    #[test]
    fn test_storage_empty_message_list() {
        let session = ChatSession::new("/test/project".to_string(), "Test Session".to_string());
        let messages = vec![];

        storage::save_session(&session, &messages).expect("Failed to save session");

        let (_, loaded_messages) = storage::load_session(&session.id).expect("Failed to load");
        assert_eq!(loaded_messages.len(), 0);

        // Cleanup
        let _ = storage::delete_session(&session.id);
    }

    // ====================================================================
    // Serialization Tests
    // ====================================================================

    #[test]
    fn test_session_serialization() {
        let session = ChatSession::new("/test/project".to_string(), "Test Session".to_string());
        let json = serde_json::to_string(&session).expect("Failed to serialize");
        let deserialized: ChatSession =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.project_path, session.project_path);
        assert_eq!(deserialized.title, session.title);
    }

    #[test]
    fn test_message_serialization() {
        let message = ChatMessage::new(
            "session-123".to_string(),
            MessageRole::User,
            "Test message".to_string(),
        );
        let json = serde_json::to_string(&message).expect("Failed to serialize");
        let deserialized: ChatMessage =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.id, message.id);
        assert_eq!(deserialized.session_id, message.session_id);
        assert_eq!(deserialized.role, message.role);
        assert_eq!(deserialized.content, message.content);
    }

    #[test]
    fn test_message_role_serialization() {
        let roles = vec![
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::System,
            MessageRole::Tool,
        ];

        for role in roles {
            let json = serde_json::to_string(&role).expect("Failed to serialize");
            let deserialized: MessageRole =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert_eq!(deserialized, role);
        }
    }

    // ====================================================================
    // Edge Cases and Error Handling
    // ====================================================================

    #[test]
    fn test_chat_message_with_special_characters() {
        let message = ChatMessage::new(
            "session-123".to_string(),
            MessageRole::User,
            "Special chars: \n\t\r\"\'\\".to_string(),
        );

        let json = serde_json::to_string(&message).expect("Failed to serialize");
        let deserialized: ChatMessage =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.content, "Special chars: \n\t\r\"\'\\");
    }

    #[test]
    fn test_chat_message_with_empty_content() {
        let message = ChatMessage::new("session-123".to_string(), MessageRole::User, "".to_string());
        assert_eq!(message.content, "");

        let json = serde_json::to_string(&message).expect("Failed to serialize");
        let deserialized: ChatMessage =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.content, "");
    }

    #[test]
    fn test_chat_session_with_long_title() {
        let long_title = "A".repeat(1000);
        let session = ChatSession::new("/test/project".to_string(), long_title.clone());
        assert_eq!(session.title, long_title);
    }

    #[test]
    fn test_storage_delete_nonexistent_session() {
        // Should not error on deleting nonexistent session (with valid UUID format)
        let nonexistent_uuid = "00000000-0000-0000-0000-000000000001";
        let result = storage::delete_session(nonexistent_uuid);
        assert!(result.is_ok(), "Delete should not error on nonexistent session");
    }
}
