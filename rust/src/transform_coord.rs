use geo_types::coord;
use shapefile::Shape;

use crate::{
    crs::{JapanCrs, PROJ4STRING_TOKYO, PROJ4STRING_WGS84},
    error::Ksj2GpError,
};

pub struct CoordTransformer {
    src: JapanCrs,
}

impl CoordTransformer {
    pub fn new(src: JapanCrs) -> Self {
        Self { src }
    }
    pub fn transform(&self, shape: &Shape) -> Result<geo_types::Geometry, Ksj2GpError> {
        match shape {
            Shape::Point(point) => Ok(geo_types::Geometry::Point(
                self.transform_single_point(point)?.into(),
            )),
            // Shape::PointZ(point) => {
            //     Ok(geojson::Value::Point(self.transform_single_point_z(point)?))
            // }
            Shape::Polyline(polyline) => {
                let linestrings = polyline
                    .parts()
                    .iter()
                    .map(|points| {
                        self.transform_points(points)
                            .map(|coords| geo_types::LineString::new(coords))
                    })
                    .collect::<Result<Vec<geo_types::LineString>, _>>()?;
                Ok(geo_types::Geometry::MultiLineString(
                    geo_types::MultiLineString::new(linestrings),
                ))
            }
            // Shape::PolylineZ(polyline) => {
            //     let positions = polyline
            //         .parts()
            //         .iter()
            //         .map(|points| self.transform_points_z(points))
            //         .collect::<Result<Vec<Vec<geojson::Position>>, _>>()?;
            //     Ok(geojson::Value::MultiLineString(positions))
            // }
            // TODO: this naively assumes the inner ring always comes first.
            // Need to check the actual specification...
            Shape::Polygon(polygon) => {
                let mut rings = polygon.rings().iter();
                let exterior = match rings.next() {
                    Some(ring) => {
                        let points = ring.points();
                        let coords = self.transform_points(points)?;
                        geo_types::LineString::new(coords)
                    }
                    None => return Err("Empty polygon".into()),
                };
                let interior = rings
                    .map(|ring| {
                        let points = ring.points();
                        match self.transform_points(points) {
                            Ok(coords) => Ok(geo_types::LineString::new(coords)),
                            Err(e) => Err(e),
                        }
                    })
                    .collect::<Result<Vec<geo_types::LineString>, _>>()?;

                Ok(geo_types::Geometry::Polygon(geo_types::Polygon::new(
                    exterior, interior,
                )))
            }

            // Shape::PolygonZ(polygon) => {
            //     let positions = polygon
            //         .rings()
            //         .iter()
            //         .map(|ring| {
            //             let points = ring.points();
            //             self.transform_points_z(points)
            //         })
            //         .collect::<Result<Vec<Vec<geojson::Position>>, _>>()?;

            //     Ok(geojson::Value::Polygon(positions))
            // }
            Shape::Multipoint(multipoint) => {
                let positions = self.transform_points(multipoint.points())?;
                Ok(geo_types::Geometry::MultiPoint(positions.into()))
            }
            // Shape::MultipointZ(multipoint) => {
            //     let positions = self.transform_points_z(multipoint.points())?;
            //     Ok(geojson::Value::MultiPoint(positions))
            // }
            _ => Err(format!("Unsupported shape type: {}", shape.shapetype()).into()),
        }
    }

    fn transform_single_point(
        &self,
        point: &shapefile::Point,
    ) -> Result<geo_types::Coord, Ksj2GpError> {
        // JGD2000, JGD2011 から WGS84 は無変換とする
        match self.src {
            JapanCrs::Tokyo => {
                // Note: proj4rs requires the longitude and latitude in radian, not in degree.
                // So, we must convert it to radians and then convert back to degree...
                let mut pt = (point.x.to_radians(), point.y.to_radians());
                proj4rs::transform::transform(&PROJ4STRING_TOKYO, &PROJ4STRING_WGS84, &mut pt)?;
                Ok(coord! { x: pt.0.to_degrees(), y: pt.1.to_degrees() })
            }
            JapanCrs::JGD2000 | JapanCrs::JGD2011 => Ok(coord! { x: point.x, y: point.y }),
        }
    }

    // fn transform_single_point_z(
    //     &self,
    //     point: &shapefile::PointZ,
    // ) -> Result<geo_types::CoordZ, Ksj2GpError> {
    //     match self.src {
    //         JapanCrs::Tokyo => {
    //             let mut pt = (
    //                 point.x.to_radians(),
    //                 point.y.to_radians(),
    //                 point.z.to_radians(),
    //             );
    //             proj4rs::transform::transform(&PROJ4STRING_TOKYO, &PROJ4STRING_WGS84, &mut pt)?;
    //             Ok(vec![
    //                 pt.0.to_degrees(),
    //                 pt.1.to_degrees(),
    //                 pt.2.to_degrees(),
    //             ])
    //         }
    //         JapanCrs::JGD2000 | JapanCrs::JGD2011 => Ok(vec![point.x, point.y, point.z]),
    //     }
    // }

    fn transform_points(
        &self,
        points: &[shapefile::Point],
    ) -> Result<Vec<geo_types::Coord>, Ksj2GpError> {
        points
            .iter()
            .map(|point| self.transform_single_point(point))
            .collect::<Result<Vec<geo_types::Coord>, _>>()
    }

    // fn transform_points_z(
    //     &self,
    //     points: &[shapefile::PointZ],
    // ) -> Result<Vec<geojson::Position>, Ksj2GpError> {
    //     points
    //         .iter()
    //         .map(|point| self.transform_single_point_z(point))
    //         .collect::<Result<Vec<geojson::Position>, _>>()
    // }
}
