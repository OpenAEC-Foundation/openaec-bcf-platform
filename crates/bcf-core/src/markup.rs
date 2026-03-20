//! BCF 2.1 `markup.bcf` XML parsing and generation.
//!
//! Each topic folder in a .bcfzip contains a `markup.bcf` file with the
//! topic metadata, comments, and viewpoint references.

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

use crate::error::BcfError;
use crate::types::{BcfComment, BcfTopic};
use crate::xml_types::{
  XmlComment, XmlCommentViewpointRef, XmlLabels, XmlMarkup, XmlTopic, XmlViewpointRef,
};

/// Reference linking a viewpoint file and snapshot within a topic folder.
#[derive(Debug, Clone)]
pub struct ViewpointRef {
  pub guid: Uuid,
  pub viewpoint_file: String,
  pub snapshot_file: Option<String>,
}

/// Parse a `markup.bcf` file into topic, comments, and viewpoint references.
pub fn parse_markup(
  xml: &[u8],
) -> Result<(BcfTopic, Vec<BcfComment>, Vec<ViewpointRef>), BcfError> {
  let markup: XmlMarkup = quick_xml::de::from_reader(xml)?;

  let topic = parse_topic(&markup.topic)?;
  let comments = markup
    .comments
    .into_iter()
    .map(|c| parse_comment(&c))
    .collect::<Result<Vec<_>, _>>()?;
  let viewpoint_refs = markup
    .viewpoints
    .into_iter()
    .map(|v| parse_viewpoint_ref(&v))
    .collect::<Result<Vec<_>, _>>()?;

  Ok((topic, comments, viewpoint_refs))
}

/// Generate a `markup.bcf` XML file from topic, comments, and viewpoint refs.
pub fn generate_markup(
  topic: &BcfTopic,
  comments: &[BcfComment],
  viewpoint_refs: &[ViewpointRef],
) -> Result<Vec<u8>, BcfError> {
  let xml_topic = topic_to_xml(topic);
  let xml_comments: Vec<XmlComment> = comments.iter().map(comment_to_xml).collect();
  let xml_viewpoints: Vec<XmlViewpointRef> =
    viewpoint_refs.iter().map(viewpoint_ref_to_xml).collect();

  let markup = XmlMarkup {
    topic: xml_topic,
    comments: xml_comments,
    viewpoints: xml_viewpoints,
  };

  let xml = quick_xml::se::to_string(&markup)
    .map_err(|e| BcfError::InvalidData(format!("XML serialization failed: {e}")))?;

  let mut output = Vec::new();
  output.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
  output.extend_from_slice(xml.as_bytes());
  Ok(output)
}

fn parse_topic(t: &XmlTopic) -> Result<BcfTopic, BcfError> {
  let guid = Uuid::parse_str(&t.guid)
    .map_err(|e| BcfError::InvalidData(format!("invalid topic GUID: {e}")))?;

  let creation_date = t
    .creation_date
    .as_deref()
    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    .map(|dt| dt.with_timezone(&Utc));

  let modified_date = t
    .modified_date
    .as_deref()
    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    .map(|dt| dt.with_timezone(&Utc));

  let due_date = t
    .due_date
    .as_deref()
    .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

  let labels = t
    .labels
    .as_ref()
    .map(|l| l.labels.clone())
    .unwrap_or_default();

  Ok(BcfTopic {
    guid,
    title: t.title.clone(),
    description: t.description.clone(),
    topic_type: t.topic_type.clone(),
    topic_status: if t.topic_status.is_empty() {
      "Open".to_string()
    } else {
      t.topic_status.clone()
    },
    priority: if t.priority.is_empty() {
      "Normal".to_string()
    } else {
      t.priority.clone()
    },
    stage: t.stage.clone().unwrap_or_default(),
    labels,
    due_date,
    assigned_to: t.assigned_to.clone(),
    creation_author: t.creation_author.clone(),
    modified_author: t.modified_author.clone(),
    creation_date,
    modified_date,
    index: t.index,
  })
}

fn parse_comment(c: &XmlComment) -> Result<BcfComment, BcfError> {
  let guid = Uuid::parse_str(&c.guid)
    .map_err(|e| BcfError::InvalidData(format!("invalid comment GUID: {e}")))?;

  let date = c
    .date
    .as_deref()
    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    .map(|dt| dt.with_timezone(&Utc));

  let modified_date = c
    .modified_date
    .as_deref()
    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    .map(|dt| dt.with_timezone(&Utc));

  let viewpoint_guid = c
    .viewpoint
    .as_ref()
    .and_then(|v| Uuid::parse_str(&v.guid).ok());

  Ok(BcfComment {
    guid,
    comment: c.comment.clone(),
    author: c.author.clone(),
    viewpoint_guid,
    date,
    modified_date,
    modified_author: c.modified_author.clone(),
  })
}

fn parse_viewpoint_ref(v: &XmlViewpointRef) -> Result<ViewpointRef, BcfError> {
  let guid = Uuid::parse_str(&v.guid)
    .map_err(|e| BcfError::InvalidData(format!("invalid viewpoint ref GUID: {e}")))?;

  Ok(ViewpointRef {
    guid,
    viewpoint_file: v.viewpoint.clone(),
    snapshot_file: v.snapshot.clone(),
  })
}

fn topic_to_xml(t: &BcfTopic) -> XmlTopic {
  let labels = if t.labels.is_empty() {
    None
  } else {
    Some(XmlLabels {
      labels: t.labels.clone(),
    })
  };

  XmlTopic {
    guid: t.guid.to_string(),
    topic_type: t.topic_type.clone(),
    topic_status: t.topic_status.clone(),
    title: t.title.clone(),
    description: t.description.clone(),
    priority: t.priority.clone(),
    index: t.index,
    creation_date: t.creation_date.map(|d| d.to_rfc3339()),
    creation_author: t.creation_author.clone(),
    modified_date: t.modified_date.map(|d| d.to_rfc3339()),
    modified_author: t.modified_author.clone(),
    assigned_to: t.assigned_to.clone(),
    stage: if t.stage.is_empty() {
      None
    } else {
      Some(t.stage.clone())
    },
    due_date: t.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
    labels,
  }
}

fn comment_to_xml(c: &BcfComment) -> XmlComment {
  XmlComment {
    guid: c.guid.to_string(),
    date: c.date.map(|d| d.to_rfc3339()),
    author: c.author.clone(),
    comment: c.comment.clone(),
    viewpoint: c.viewpoint_guid.map(|g| XmlCommentViewpointRef {
      guid: g.to_string(),
    }),
    modified_date: c.modified_date.map(|d| d.to_rfc3339()),
    modified_author: c.modified_author.clone(),
  }
}

fn viewpoint_ref_to_xml(v: &ViewpointRef) -> XmlViewpointRef {
  XmlViewpointRef {
    guid: v.guid.to_string(),
    viewpoint: v.viewpoint_file.clone(),
    snapshot: v.snapshot_file.clone(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const MARKUP_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Markup>
  <Topic Guid="d1d2d3d4-d5d6-d7d8-d9da-dbdcdddedfe0" TopicType="Error" TopicStatus="Open">
    <Title>Missing wall</Title>
    <Description>Wall is missing in model</Description>
    <Priority>Critical</Priority>
    <Index>1</Index>
    <CreationDate>2024-01-15T10:30:00+00:00</CreationDate>
    <CreationAuthor>architect@example.com</CreationAuthor>
    <Labels>
      <Label>Architecture</Label>
      <Label>Structural</Label>
    </Labels>
  </Topic>
  <Comment Guid="e1e2e3e4-e5e6-e7e8-e9ea-ebecedeeeff0">
    <Date>2024-01-15T11:00:00+00:00</Date>
    <Author>reviewer@example.com</Author>
    <Comment>Please fix this ASAP</Comment>
    <Viewpoint Guid="f1f2f3f4-f5f6-f7f8-f9fa-fbfcfdfeff00"/>
  </Comment>
  <Viewpoints Guid="f1f2f3f4-f5f6-f7f8-f9fa-fbfcfdfeff00">
    <Viewpoint>f1f2f3f4-f5f6-f7f8-f9fa-fbfcfdfeff00.bcfv</Viewpoint>
    <Snapshot>snapshot.png</Snapshot>
  </Viewpoints>
</Markup>"#;

  #[test]
  fn parse_markup_full() {
    let (topic, comments, vp_refs) = parse_markup(MARKUP_XML.as_bytes()).unwrap();

    assert_eq!(topic.title, "Missing wall");
    assert_eq!(topic.topic_type, "Error");
    assert_eq!(topic.topic_status, "Open");
    assert_eq!(topic.priority, "Critical");
    assert_eq!(topic.index, Some(1));
    assert_eq!(topic.labels, vec!["Architecture", "Structural"]);
    assert_eq!(
      topic.creation_author.as_deref(),
      Some("architect@example.com")
    );

    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].comment, "Please fix this ASAP");
    assert_eq!(comments[0].author.as_deref(), Some("reviewer@example.com"));
    assert!(comments[0].viewpoint_guid.is_some());

    assert_eq!(vp_refs.len(), 1);
    assert_eq!(
      vp_refs[0].viewpoint_file,
      "f1f2f3f4-f5f6-f7f8-f9fa-fbfcfdfeff00.bcfv"
    );
    assert_eq!(vp_refs[0].snapshot_file.as_deref(), Some("snapshot.png"));
  }

  #[test]
  fn roundtrip_markup() {
    let (topic, comments, vp_refs) = parse_markup(MARKUP_XML.as_bytes()).unwrap();

    let xml = generate_markup(&topic, &comments, &vp_refs).unwrap();
    let (topic2, comments2, vp_refs2) = parse_markup(&xml).unwrap();

    assert_eq!(topic2.title, topic.title);
    assert_eq!(topic2.topic_type, topic.topic_type);
    assert_eq!(topic2.labels, topic.labels);
    assert_eq!(comments2.len(), comments.len());
    assert_eq!(comments2[0].comment, comments[0].comment);
    assert_eq!(vp_refs2.len(), vp_refs.len());
    assert_eq!(vp_refs2[0].viewpoint_file, vp_refs[0].viewpoint_file);
  }

  #[test]
  fn parse_minimal_markup() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Markup>
  <Topic Guid="a0a1a2a3-a4a5-a6a7-a8a9-aaabacadaeaf">
    <Title>Simple topic</Title>
  </Topic>
</Markup>"#;

    let (topic, comments, vp_refs) = parse_markup(xml.as_bytes()).unwrap();
    assert_eq!(topic.title, "Simple topic");
    assert_eq!(topic.topic_status, "Open");
    assert_eq!(topic.priority, "Normal");
    assert!(comments.is_empty());
    assert!(vp_refs.is_empty());
  }
}
