//! Notification template engine

use crate::error::{NotificationError, Result};
use crate::types::Notification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::sync::RwLock;

/// Notification template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub title_template: String,
    pub message_template: String,
    pub html_template: Option<String>,
    pub default_vars: HashMap<String, serde_json::Value>,
}

/// Template engine
pub struct TemplateEngine {
    tera: Arc<RwLock<Tera>>,
    templates: Arc<RwLock<HashMap<String, NotificationTemplate>>>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new(templates_dir: Option<&str>) -> Result<Self> {
        let tera = if let Some(dir) = templates_dir {
            Tera::new(&format!("{}/**/*.html", dir))
                .map_err(|e| NotificationError::Template(format!("Failed to load templates: {}", e)))?
        } else {
            Tera::default()
        };

        Ok(Self {
            tera: Arc::new(RwLock::new(tera)),
            templates: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a template
    pub async fn register_template(&self, template: NotificationTemplate) -> Result<()> {
        // Add templates to Tera
        let mut tera = self.tera.write().await;

        tera.add_raw_template(
            &format!("{}_title", template.id),
            &template.title_template,
        )
        .map_err(|e| NotificationError::Template(format!("Invalid title template: {}", e)))?;

        tera.add_raw_template(
            &format!("{}_message", template.id),
            &template.message_template,
        )
        .map_err(|e| NotificationError::Template(format!("Invalid message template: {}", e)))?;

        if let Some(html) = &template.html_template {
            tera.add_raw_template(&format!("{}_html", template.id), html)
                .map_err(|e| NotificationError::Template(format!("Invalid HTML template: {}", e)))?;
        }

        drop(tera);

        // Store template
        self.templates
            .write()
            .await
            .insert(template.id.clone(), template);

        Ok(())
    }

    /// Render a template
    pub async fn render(
        &self,
        template_id: &str,
        vars: &HashMap<String, serde_json::Value>,
    ) -> Result<RenderedTemplate> {
        let template = self
            .templates
            .read()
            .await
            .get(template_id)
            .cloned()
            .ok_or_else(|| NotificationError::Template(format!("Template not found: {}", template_id)))?;

        // Merge default vars with provided vars
        let mut context = Context::new();
        for (key, value) in &template.default_vars {
            context.insert(key, value);
        }
        for (key, value) in vars {
            context.insert(key, value);
        }

        let tera = self.tera.read().await;

        // Render title
        let title = tera
            .render(&format!("{}_title", template_id), &context)
            .map_err(|e| NotificationError::Template(format!("Failed to render title: {}", e)))?;

        // Render message
        let message = tera
            .render(&format!("{}_message", template_id), &context)
            .map_err(|e| NotificationError::Template(format!("Failed to render message: {}", e)))?;

        // Render HTML if available
        let html_message = if template.html_template.is_some() {
            Some(
                tera.render(&format!("{}_html", template_id), &context)
                    .map_err(|e| NotificationError::Template(format!("Failed to render HTML: {}", e)))?,
            )
        } else {
            None
        };

        Ok(RenderedTemplate {
            title,
            message,
            html_message,
        })
    }

    /// Apply template to a notification
    pub async fn apply_template(
        &self,
        notification: &mut Notification,
        template_id: &str,
    ) -> Result<()> {
        let rendered = self.render(template_id, &notification.template_vars).await?;

        notification.title = rendered.title;
        notification.message = rendered.message;
        notification.html_message = rendered.html_message;
        notification.template_id = Some(template_id.to_string());

        Ok(())
    }

    /// Get template
    pub async fn get_template(&self, template_id: &str) -> Option<NotificationTemplate> {
        self.templates.read().await.get(template_id).cloned()
    }

    /// List all templates
    pub async fn list_templates(&self) -> Vec<NotificationTemplate> {
        self.templates.read().await.values().cloned().collect()
    }

    /// Remove template
    pub async fn remove_template(&self, template_id: &str) -> Result<()> {
        let mut tera = self.tera.write().await;

        // Remove from Tera (note: Tera doesn't have a direct remove method, so we'll just keep them)
        // In a production system, you might want to rebuild Tera without these templates

        drop(tera);

        self.templates.write().await.remove(template_id);
        Ok(())
    }

    /// Register built-in templates
    pub async fn register_builtin_templates(&self) -> Result<()> {
        // Welcome template
        self.register_template(NotificationTemplate {
            id: "welcome".to_string(),
            name: "Welcome".to_string(),
            description: "Welcome new users".to_string(),
            title_template: "Welcome to AccuScene, {{ user_name }}!".to_string(),
            message_template: "We're excited to have you on board. Get started by creating your first case.".to_string(),
            html_template: Some(
                r#"<h1>Welcome to AccuScene, {{ user_name }}!</h1>
                <p>We're excited to have you on board.</p>
                <p>Get started by creating your first case.</p>"#.to_string(),
            ),
            default_vars: HashMap::new(),
        })
        .await?;

        // Case assignment template
        self.register_template(NotificationTemplate {
            id: "case_assigned".to_string(),
            name: "Case Assigned".to_string(),
            description: "Notify user of case assignment".to_string(),
            title_template: "New case assigned: {{ case_name }}".to_string(),
            message_template: "You have been assigned to case '{{ case_name }}' by {{ assigned_by }}.".to_string(),
            html_template: Some(
                r#"<h2>New Case Assignment</h2>
                <p>You have been assigned to case <strong>{{ case_name }}</strong> by {{ assigned_by }}.</p>
                <p><a href="{{ case_url }}">View Case</a></p>"#.to_string(),
            ),
            default_vars: HashMap::new(),
        })
        .await?;

        // Report ready template
        self.register_template(NotificationTemplate {
            id: "report_ready".to_string(),
            name: "Report Ready".to_string(),
            description: "Notify user that report is ready".to_string(),
            title_template: "Report ready: {{ report_name }}".to_string(),
            message_template: "Your report '{{ report_name }}' has been generated and is ready for download.".to_string(),
            html_template: Some(
                r#"<h2>Report Ready</h2>
                <p>Your report <strong>{{ report_name }}</strong> has been generated and is ready for download.</p>
                <p><a href="{{ report_url }}">Download Report</a></p>"#.to_string(),
            ),
            default_vars: HashMap::new(),
        })
        .await?;

        // Comment mention template
        self.register_template(NotificationTemplate {
            id: "comment_mention".to_string(),
            name: "Comment Mention".to_string(),
            description: "Notify user of mention in comment".to_string(),
            title_template: "{{ author }} mentioned you in a comment".to_string(),
            message_template: "{{ author }} mentioned you: {{ comment_preview }}".to_string(),
            html_template: Some(
                r#"<h2>You were mentioned</h2>
                <p><strong>{{ author }}</strong> mentioned you in a comment:</p>
                <blockquote>{{ comment_preview }}</blockquote>
                <p><a href="{{ comment_url }}">View Comment</a></p>"#.to_string(),
            ),
            default_vars: HashMap::new(),
        })
        .await?;

        // Analysis complete template
        self.register_template(NotificationTemplate {
            id: "analysis_complete".to_string(),
            name: "Analysis Complete".to_string(),
            description: "Notify user that analysis is complete".to_string(),
            title_template: "Analysis complete: {{ analysis_type }}".to_string(),
            message_template: "Your {{ analysis_type }} analysis has completed successfully.".to_string(),
            html_template: Some(
                r#"<h2>Analysis Complete</h2>
                <p>Your <strong>{{ analysis_type }}</strong> analysis has completed successfully.</p>
                <p><a href="{{ results_url }}">View Results</a></p>"#.to_string(),
            ),
            default_vars: HashMap::new(),
        })
        .await?;

        // Security alert template
        self.register_template(NotificationTemplate {
            id: "security_alert".to_string(),
            name: "Security Alert".to_string(),
            description: "Security-related alerts".to_string(),
            title_template: "Security Alert: {{ alert_type }}".to_string(),
            message_template: "{{ alert_message }}".to_string(),
            html_template: Some(
                r#"<h2 style="color: red;">Security Alert: {{ alert_type }}</h2>
                <p>{{ alert_message }}</p>
                <p><a href="{{ action_url }}">Take Action</a></p>"#.to_string(),
            ),
            default_vars: HashMap::new(),
        })
        .await?;

        Ok(())
    }
}

/// Rendered template result
#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    pub title: String,
    pub message: String,
    pub html_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_template_rendering() {
        let engine = TemplateEngine::new(None).unwrap();

        let template = NotificationTemplate {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            title_template: "Hello {{ name }}!".to_string(),
            message_template: "Welcome {{ name }}, you have {{ count }} messages.".to_string(),
            html_template: None,
            default_vars: HashMap::new(),
        };

        engine.register_template(template).await.unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), serde_json::json!("Alice"));
        vars.insert("count".to_string(), serde_json::json!(5));

        let rendered = engine.render("test", &vars).await.unwrap();

        assert_eq!(rendered.title, "Hello Alice!");
        assert_eq!(rendered.message, "Welcome Alice, you have 5 messages.");
    }
}
