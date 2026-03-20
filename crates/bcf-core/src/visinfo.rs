//! Viewpoint BCF v2.1 XML parsing and generation.
//!
//! Handles `viewpoint.bcfv` files — the `<VisualizationInfo>` root element
//! containing camera position and component visibility/selection/coloring.

use uuid::Uuid;

use crate::error::BcfError;
use crate::types::{BcfViewpoint, Camera, Components};
use crate::xml_types::{
  XmlOrthogonalCamera, XmlPerspectiveCamera, XmlPoint3D, XmlVisualizationInfo,
};

/// Parse a `viewpoint.bcfv` XML file into a [`BcfViewpoint`].
pub fn parse_visinfo(xml: &[u8]) -> Result<BcfViewpoint, BcfError> {
  let info: XmlVisualizationInfo = quick_xml::de::from_reader(xml)?;

  let guid = Uuid::parse_str(&info.guid)
    .map_err(|e| BcfError::InvalidData(format!("invalid viewpoint GUID: {e}")))?;

  let camera = if let Some(pc) = info.perspective_camera {
    Some(Camera::from(pc))
  } else {
    info.orthogonal_camera.map(Camera::from)
  };

  let components = info.components.map(Components::from);

  Ok(BcfViewpoint {
    guid,
    camera,
    components,
    snapshot_data: None,
  })
}

/// Generate a `viewpoint.bcfv` XML file from a [`BcfViewpoint`].
pub fn generate_visinfo(viewpoint: &BcfViewpoint) -> Result<Vec<u8>, BcfError> {
  let (perspective_camera, orthogonal_camera) = match &viewpoint.camera {
    Some(cam) if cam.camera_type == "orthogonal" => {
      let oc = XmlOrthogonalCamera {
        camera_view_point: XmlPoint3D::from(&cam.position),
        camera_direction: XmlPoint3D::from(&cam.direction),
        camera_up_vector: XmlPoint3D::from(&cam.up),
        view_to_world_scale: cam.field_of_view.unwrap_or(1.0),
        aspect_ratio: cam.aspect_ratio.unwrap_or(1.0),
      };
      (None, Some(oc))
    }
    Some(cam) => {
      let pc = XmlPerspectiveCamera {
        camera_view_point: XmlPoint3D::from(&cam.position),
        camera_direction: XmlPoint3D::from(&cam.direction),
        camera_up_vector: XmlPoint3D::from(&cam.up),
        field_of_view: cam.field_of_view.unwrap_or(60.0),
        aspect_ratio: cam.aspect_ratio.unwrap_or(1.0),
      };
      (Some(pc), None)
    }
    None => (None, None),
  };

  let components = viewpoint
    .components
    .as_ref()
    .map(crate::xml_types::XmlComponents::from);

  let info = XmlVisualizationInfo {
    guid: viewpoint.guid.to_string(),
    perspective_camera,
    orthogonal_camera,
    components,
  };

  let xml = quick_xml::se::to_string(&info)
    .map_err(|e| BcfError::InvalidData(format!("XML serialization failed: {e}")))?;

  let mut output = Vec::new();
  output.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
  output.extend_from_slice(xml.as_bytes());
  Ok(output)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::Point3D;

  const PERSPECTIVE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VisualizationInfo Guid="b0b1b2b3-b4b5-b6b7-b8b9-babbbcbdbebf">
  <PerspectiveCamera>
    <CameraViewPoint><X>10.0</X><Y>20.0</Y><Z>30.0</Z></CameraViewPoint>
    <CameraDirection><X>0.0</X><Y>0.0</Y><Z>-1.0</Z></CameraDirection>
    <CameraUpVector><X>0.0</X><Y>1.0</Y><Z>0.0</Z></CameraUpVector>
    <FieldOfView>60.0</FieldOfView>
    <AspectRatio>1.777</AspectRatio>
  </PerspectiveCamera>
</VisualizationInfo>"#;

  const ORTHOGONAL_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VisualizationInfo Guid="a1a2a3a4-a5a6-a7a8-a9aa-abacadaeafb0">
  <OrthogonalCamera>
    <CameraViewPoint><X>5.0</X><Y>5.0</Y><Z>5.0</Z></CameraViewPoint>
    <CameraDirection><X>-1.0</X><Y>0.0</Y><Z>0.0</Z></CameraDirection>
    <CameraUpVector><X>0.0</X><Y>0.0</Y><Z>1.0</Z></CameraUpVector>
    <ViewToWorldScale>100.0</ViewToWorldScale>
    <AspectRatio>1.5</AspectRatio>
  </OrthogonalCamera>
</VisualizationInfo>"#;

  #[test]
  fn parse_perspective_camera() {
    let vp = parse_visinfo(PERSPECTIVE_XML.as_bytes()).unwrap();
    assert_eq!(
      vp.guid.to_string(),
      "b0b1b2b3-b4b5-b6b7-b8b9-babbbcbdbebf"
    );
    let cam = vp.camera.unwrap();
    assert_eq!(cam.camera_type, "perspective");
    assert_eq!(cam.position.x, 10.0);
    assert_eq!(cam.field_of_view, Some(60.0));
    assert_eq!(cam.aspect_ratio, Some(1.777));
  }

  #[test]
  fn parse_orthogonal_camera() {
    let vp = parse_visinfo(ORTHOGONAL_XML.as_bytes()).unwrap();
    let cam = vp.camera.unwrap();
    assert_eq!(cam.camera_type, "orthogonal");
    assert_eq!(cam.position.x, 5.0);
    assert_eq!(cam.field_of_view, Some(100.0));
  }

  #[test]
  fn roundtrip_perspective() {
    let vp = BcfViewpoint {
      guid: Uuid::parse_str("b0b1b2b3-b4b5-b6b7-b8b9-babbbcbdbebf").unwrap(),
      camera: Some(Camera {
        camera_type: "perspective".to_string(),
        position: Point3D {
          x: 1.0,
          y: 2.0,
          z: 3.0,
        },
        direction: Point3D {
          x: 0.0,
          y: 0.0,
          z: -1.0,
        },
        up: Point3D {
          x: 0.0,
          y: 1.0,
          z: 0.0,
        },
        field_of_view: Some(60.0),
        aspect_ratio: Some(1.777),
      }),
      components: None,
      snapshot_data: None,
    };

    let xml = generate_visinfo(&vp).unwrap();
    let parsed = parse_visinfo(&xml).unwrap();

    assert_eq!(parsed.guid, vp.guid);
    let cam = parsed.camera.unwrap();
    assert_eq!(cam.camera_type, "perspective");
    assert_eq!(cam.position.x, 1.0);
    assert_eq!(cam.field_of_view, Some(60.0));
  }

  #[test]
  fn parse_with_components() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<VisualizationInfo Guid="c1c2c3c4-c5c6-c7c8-c9ca-cbcccdcecfd0">
  <Components>
    <Visibility DefaultVisibility="false">
      <Exceptions>
        <Component IfcGuid="2MF88YJnv9OOVFC8W7Swll">
          <OriginatingSystem>Revit</OriginatingSystem>
          <AuthoringToolId>123</AuthoringToolId>
        </Component>
      </Exceptions>
    </Visibility>
    <Selection>
      <Component IfcGuid="3AB12CDef4GHIJ5678Klmn"/>
    </Selection>
  </Components>
</VisualizationInfo>"#;

    let vp = parse_visinfo(xml.as_bytes()).unwrap();
    let comp = vp.components.unwrap();
    assert!(!comp.visibility.default_visibility);
    assert_eq!(comp.visibility.exceptions.len(), 1);
    assert_eq!(
      comp.visibility.exceptions[0].ifc_guid,
      "2MF88YJnv9OOVFC8W7Swll"
    );
    assert_eq!(comp.selection.len(), 1);
  }
}
