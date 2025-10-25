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
    pub fn transform(
        &self,
        shape: &Shape,
    ) -> Result<geo_traits::structs::Geometry<f64>, Ksj2GpError> {
        match shape {
            Shape::Point(point) => {
                let coord = self.transform_single_point(point)?;
                Ok(geo_traits::structs::Geometry::Point(
                    geo_traits::structs::Point::from_coord(coord),
                ))
            }
            Shape::PointZ(point) => {
                let coord = self.transform_single_point_z(point)?;
                Ok(geo_traits::structs::Geometry::Point(
                    geo_traits::structs::Point::from_coord(coord),
                ))
            }
            Shape::Polyline(polyline) => {
                let linestrings = polyline
                    .parts()
                    .iter()
                    .map(|points| {
                        self.transform_points(points).map(|coords| {
                            geo_traits::structs::LineString::from_coords(coords).unwrap()
                        })
                    })
                    .collect::<Result<Vec<geo_traits::structs::LineString>, _>>()?;
                Ok(geo_traits::structs::Geometry::MultiLineString(
                    geo_traits::structs::MultiLineString::from_line_strings(linestrings).unwrap(),
                ))
            }
            Shape::PolylineZ(polyline) => {
                let linestrings = polyline
                    .parts()
                    .iter()
                    .map(|points| {
                        self.transform_points_z(points).map(|coords| {
                            geo_traits::structs::LineString::from_coords(coords).unwrap()
                        })
                    })
                    .collect::<Result<Vec<geo_traits::structs::LineString>, _>>()?;
                Ok(geo_traits::structs::Geometry::MultiLineString(
                    geo_traits::structs::MultiLineString::from_line_strings(linestrings).unwrap(),
                ))
            }
            Shape::Polygon(polygon) => {
                let rings = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let points = ring.points();
                        match self.transform_points(points) {
                            Ok(coords) => {
                                Ok(geo_traits::structs::LineString::from_coords(coords).unwrap())
                            }
                            Err(e) => Err(e),
                        }
                    })
                    .collect::<Result<Vec<geo_traits::structs::LineString>, _>>()?;

                Ok(geo_traits::structs::Geometry::Polygon(
                    geo_traits::structs::Polygon::from_rings(rings).unwrap(),
                ))
            }
            Shape::PolygonZ(polygon) => {
                let rings = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let points = ring.points();
                        match self.transform_points_z(points) {
                            Ok(coords) => {
                                Ok(geo_traits::structs::LineString::from_coords(coords).unwrap())
                            }
                            Err(e) => Err(e),
                        }
                    })
                    .collect::<Result<Vec<geo_traits::structs::LineString>, _>>()?;

                Ok(geo_traits::structs::Geometry::Polygon(
                    geo_traits::structs::Polygon::from_rings(rings).unwrap(),
                ))
            }
            Shape::Multipoint(multipoint) => {
                let points = self
                    .transform_points(multipoint.points())?
                    .into_iter()
                    .map(|coord| geo_traits::structs::Point::from_coord(coord));
                Ok(geo_traits::structs::Geometry::MultiPoint(
                    geo_traits::structs::MultiPoint::from_points(points).unwrap(),
                ))
            }
            Shape::MultipointZ(multipoint) => {
                let points = self
                    .transform_points_z(multipoint.points())?
                    .into_iter()
                    .map(|coord| geo_traits::structs::Point::from_coord(coord));
                Ok(geo_traits::structs::Geometry::MultiPoint(
                    geo_traits::structs::MultiPoint::from_points(points).unwrap(),
                ))
            }
            _ => Err(format!("Unsupported shape type: {}", shape.shapetype()).into()),
        }
    }

    fn transform_single_point(
        &self,
        point: &shapefile::Point,
    ) -> Result<geo_traits::structs::Coord, Ksj2GpError> {
        // JGD2000, JGD2011 から WGS84 は無変換とする
        match self.src {
            JapanCrs::Tokyo => {
                // Note: proj4rs requires the longitude and latitude in radian, not in degree.
                // So, we must convert it to radians and then convert back to degree...
                let mut pt = (point.x.to_radians(), point.y.to_radians());
                proj4rs::transform::transform(&PROJ4STRING_TOKYO, &PROJ4STRING_WGS84, &mut pt)?;
                Ok(geo_traits::structs::Coord {
                    x: pt.0.to_degrees(),
                    y: pt.1.to_degrees(),
                    z: None,
                    m: None,
                })
            }
            JapanCrs::JGD2000 | JapanCrs::JGD2011 => Ok(geo_traits::structs::Coord {
                x: point.x,
                y: point.y,
                z: None,
                m: None,
            }),
        }
    }

    fn transform_single_point_z(
        &self,
        point: &shapefile::PointZ,
    ) -> Result<geo_traits::structs::Coord, Ksj2GpError> {
        match self.src {
            JapanCrs::Tokyo => {
                let mut pt = (
                    point.x.to_radians(),
                    point.y.to_radians(),
                    point.z.to_radians(),
                );
                proj4rs::transform::transform(&PROJ4STRING_TOKYO, &PROJ4STRING_WGS84, &mut pt)?;
                Ok(geo_traits::structs::Coord {
                    x: pt.0.to_degrees(),
                    y: pt.1.to_degrees(),
                    z: Some(pt.2.to_degrees()),
                    m: None,
                })
            }
            JapanCrs::JGD2000 | JapanCrs::JGD2011 => Ok(geo_traits::structs::Coord {
                x: point.x,
                y: point.y,
                z: Some(point.z),
                m: None,
            }),
        }
    }

    fn transform_points(
        &self,
        points: &[shapefile::Point],
    ) -> Result<Vec<geo_traits::structs::Coord>, Ksj2GpError> {
        points
            .iter()
            .map(|point| self.transform_single_point(point))
            .collect::<Result<Vec<geo_traits::structs::Coord>, _>>()
    }

    fn transform_points_z(
        &self,
        points: &[shapefile::PointZ],
    ) -> Result<Vec<geo_traits::structs::Coord>, Ksj2GpError> {
        points
            .iter()
            .map(|point| self.transform_single_point_z(point))
            .collect::<Result<Vec<geo_traits::structs::Coord>, _>>()
    }
}
