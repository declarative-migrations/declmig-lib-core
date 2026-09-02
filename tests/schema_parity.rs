use declmig_lib_core::schema_parity::{
    DatabaseEngine, DifferenceCode, ProjectionSource, compare, parse_diesel_schema,
    parse_seaorm_entity,
};

const DIESEL: &str = r#"
diesel::table! {
    use diesel::sql_types::*;

    public.bug_reports (tenant_id, id) {
        tenant_id -> Uuid,
        id -> Uuid,
        title -> Text,
        affected_users -> Int8,
        severity -> Nullable<Float8>,
        evidence -> Jsonb,
        embedding -> Vector,
    }
}
"#;

const SEAORM: &str = r#"
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bug_reports", schema_name = "public")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    pub affected_users: i64,
    pub severity: Option<f64>,
    #[sea_orm(column_type = "JsonBinary")]
    pub evidence: Json,
    #[sea_orm(column_type = "Custom(\"vector\".into())")]
    pub embedding: Vec<f32>,
}
"#;

#[test]
fn diesel_and_seaorm_structural_projections_match() {
    let diesel = parse_diesel_schema(DIESEL, DatabaseEngine::PostgreSql, "public", "2.3.12")
        .expect("Diesel fixture");
    let table = parse_seaorm_entity(SEAORM, "public")
        .expect("SeaORM parse")
        .expect("SeaORM entity");
    let mut seaorm = diesel.clone();
    seaorm.source = ProjectionSource::SeaOrmEntity;
    seaorm.generator.name = "sea-orm-cli-generate-entity".to_owned();
    seaorm.generator.version = "2.0.2".to_owned();
    seaorm.tables = vec![table];

    let report = compare(diesel, seaorm).expect("comparison");
    assert!(report.compatible, "{:#?}", report.differences);
    assert!(report.warnings.iter().any(|warning| warning.code == "vector-dimensions-not-proven"));
}

#[test]
fn nullability_and_type_drift_are_deterministic_failures() {
    let diesel = parse_diesel_schema(DIESEL, DatabaseEngine::PostgreSql, "public", "2.3.12")
        .expect("Diesel fixture");
    let table = parse_seaorm_entity(SEAORM, "public")
        .expect("SeaORM parse")
        .expect("SeaORM entity");
    let mut seaorm = diesel.clone();
    seaorm.source = ProjectionSource::SeaOrmEntity;
    seaorm.tables = vec![table];
    seaorm.tables[0].columns[2].nullable = true;
    seaorm.tables[0].columns[3].type_family = "int4".to_owned();

    let report = compare(diesel, seaorm).expect("comparison");
    assert!(!report.compatible);
    let codes = report
        .differences
        .iter()
        .map(|difference| difference.code)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        codes,
        [DifferenceCode::NullabilityMismatch, DifferenceCode::TypeFamilyMismatch]
            .into_iter()
            .collect()
    );
}
