use shapefile::Shape;

use crate::error::Ksj2GpError;

pub struct CoordTransformer {}

impl CoordTransformer {
    pub fn new() -> Self {
        Self {}
    }
    pub fn transform_to_geojson(&self, shape: &Shape) -> Result<geojson::Value, Ksj2GpError> {
        match shape {
            Shape::Point(point) => Ok(geojson::Value::Point(self.transform_single_point(point)?)),
            Shape::PointZ(point) => {
                Ok(geojson::Value::Point(self.transform_single_point_z(point)?))
            }
            Shape::Polyline(polyline) => {
                let positions = polyline
                    .parts()
                    .iter()
                    .map(|points| self.transform_points(points))
                    .collect::<Result<Vec<Vec<geojson::Position>>, _>>()?;
                Ok(geojson::Value::MultiLineString(positions))
            }
            Shape::PolylineZ(polyline) => {
                let positions = polyline
                    .parts()
                    .iter()
                    .map(|points| self.transform_points_z(points))
                    .collect::<Result<Vec<Vec<geojson::Position>>, _>>()?;
                Ok(geojson::Value::MultiLineString(positions))
            }
            // TODO: this naively assumes the inner ring always comes first.
            // Need to check the actual specification...
            Shape::Polygon(polygon) => {
                let positions = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let points = ring.points();
                        self.transform_points(points)
                    })
                    .collect::<Result<Vec<Vec<geojson::Position>>, _>>()?;

                Ok(geojson::Value::Polygon(positions))
            }

            Shape::PolygonZ(polygon) => {
                let positions = polygon
                    .rings()
                    .iter()
                    .map(|ring| {
                        let points = ring.points();
                        self.transform_points_z(points)
                    })
                    .collect::<Result<Vec<Vec<geojson::Position>>, _>>()?;

                Ok(geojson::Value::Polygon(positions))
            }
            Shape::Multipoint(multipoint) => {
                let positions = self.transform_points(multipoint.points())?;
                Ok(geojson::Value::MultiPoint(positions))
            }
            Shape::MultipointZ(multipoint) => {
                let positions = self.transform_points_z(multipoint.points())?;
                Ok(geojson::Value::MultiPoint(positions))
            }
            _ => Err(format!("Unsupported shape type: {}", shape.shapetype()).into()),
        }
    }

    fn transform_single_point(
        &self,
        point: &shapefile::Point,
    ) -> Result<geojson::Position, Ksj2GpError> {
        // TODO: add PatchJDG transformation here

        Ok(vec![point.x, point.y])
    }

    fn transform_single_point_z(
        &self,
        point: &shapefile::PointZ,
    ) -> Result<geojson::Position, Ksj2GpError> {
        // TODO: add PatchJDG transformation here

        Ok(vec![point.x, point.y, point.z])
    }

    fn transform_points(
        &self,
        points: &[shapefile::Point],
    ) -> Result<Vec<geojson::Position>, Ksj2GpError> {
        points
            .iter()
            .map(|point| self.transform_single_point(point))
            .collect::<Result<Vec<geojson::Position>, _>>()
    }

    fn transform_points_z(
        &self,
        points: &[shapefile::PointZ],
    ) -> Result<Vec<geojson::Position>, Ksj2GpError> {
        points
            .iter()
            .map(|point| self.transform_single_point_z(point))
            .collect::<Result<Vec<geojson::Position>, _>>()
    }
}
