//! JSON transport over the accepted owner's libraries; no local formal algebra.
use ql_core::{
    apply_operator, build_d_modulation_frame, canonical_cross_pass_d1, canonical_cross_pass_d2,
    canonical_cross_pass_d3, classify_relation_pair, CanonicalCrossPass, ConjugationDegree,
    D2CrossPassKind, ExpansionSide, QlAddress, QlCoordinate, QlFace, QlOperator, QlPosition,
    RelationFamily,
};
use ql_mef::{
    cross_interval_deltas, derive_pre_m_music, pair_interval_deltas, CrossOperator, MusicalBasis,
};
use serde_json::{json, Value};
use std::io::{self, Read};

const OWNER_REVISION: &str = "e753efc91f62b5b2af09e0a852c5063e366eccbe";
const SCHEMA: &str = "actuation.ql-owner-operation/v1";
const OPERATIONS: &[&str] = &[
    "capabilities",
    "vocabulary",
    "classify-relation",
    "modulation",
    "cross",
    "kernel",
    "harmonic-snapshot",
];
type Result<T> = std::result::Result<T, String>;
fn number(v: &Value, key: &str) -> Result<u8> {
    v[key]
        .as_u64()
        .and_then(|n| u8::try_from(n).ok())
        .ok_or_else(|| format!("{key} requires a u8"))
}
fn position(v: &Value, key: &str) -> Result<QlPosition> {
    QlPosition::new(number(v, key)?).map_err(|e| e.to_string())
}
fn family(v: &Value) -> Result<RelationFamily> {
    match v["family"].as_str() {
        Some("A") => Ok(RelationFamily::A),
        Some("B") => Ok(RelationFamily::B),
        Some("C") => Ok(RelationFamily::C),
        _ => Err("unknown relation family".into()),
    }
}
fn face(v: &Value) -> Result<QlFace> {
    match v["face"].as_str() {
        Some("direct") => Ok(QlFace::Direct),
        Some("conjugate") => Ok(QlFace::Conjugate),
        _ => Err("unknown face".into()),
    }
}
fn coordinate(c: QlCoordinate) -> Value {
    json!({"position":c.position.value(),"face":c.face.as_str()})
}
fn cross(c: CanonicalCrossPass) -> Value {
    let coords = match &c {
        CanonicalCrossPass::D1 { coordinates, .. } | CanonicalCrossPass::D2 { coordinates, .. } => {
            json!(coordinates
                .iter()
                .copied()
                .map(coordinate)
                .collect::<Vec<_>>())
        }
        CanonicalCrossPass::D3 { pairs, .. } => json!(pairs
            .iter()
            .map(|p| p.iter().copied().map(coordinate).collect::<Vec<_>>())
            .collect::<Vec<_>>()),
    };
    json!({"operator_ref":c.operator_ref(),"derivation_ref":c.derivation_ref(),"coordinates":coords})
}
fn invoke(v: &Value) -> Result<Value> {
    let operation = v["operation"].as_str().ok_or("operation is required")?;
    let result=match operation {
        "capabilities"=>json!({"operations":OPERATIONS,"formal_owner":"EpiLogos/QL-MEF","evidence_standing":"D","provider_evidence":false}),
        "vocabulary"=>{
            // Enumerate through the owner's validated constructor, not a second
            // privately maintained list of valid positions or class algebra.
            let positions=(u8::MIN..=u8::MAX).filter_map(|n|QlPosition::new(n).ok()).map(|p| {
                let a=QlAddress::sixfold(p.value(),QlFace::Direct,0).expect("validated position");
                let class=apply_operator(QlOperator::ClassifyFourPlusTwo,a);
                json!({"position":p.value(),"complement":p.complement().value(),"class":class.value.to_string()})
            }).collect::<Vec<_>>();
            let families=[RelationFamily::A,RelationFamily::B,RelationFamily::C].iter().map(|f|{
                let pairs=f.pairs().iter().enumerate().map(|(index,_)|{
                    let p=f.pair(index as u8).expect("owner pair");
                    json!({"index":p.pair_index,"left":p.left.value(),"right":p.right.value(),"operator_ref":p.operator_ref()})
                }).collect::<Vec<_>>(); json!({"family":f.as_str(),"pairs":pairs})
            }).collect::<Vec<_>>();
            json!({"positions":positions,"faces":[QlFace::Direct.as_str(),QlFace::Conjugate.as_str()],"families":families})
        },
        "classify-relation"=>json!(classify_relation_pair(position(v,"from")?,position(v,"to")?).iter().map(|m|json!({"family":m.family.as_str(),"pair_index":m.pair_index,"reversed":m.reversed,"operator_ref":m.operator_ref()})).collect::<Vec<_>>()),
        "modulation"=>{
            let degree=match v["degree"].as_str(){Some("D1")=>ConjugationDegree::D1,Some("D2")=>ConjugationDegree::D2,Some("D3")=>ConjugationDegree::D3,_=>return Err("unknown degree".into())};
            let side=match v["projection_side"].as_str(){None if v["projection_side"].is_null()=>None,Some("left")=>Some(ExpansionSide::Left),Some("right")=>Some(ExpansionSide::Right),_=>return Err("unknown projection side".into())};
            let f=build_d_modulation_frame(family(v)?,number(v,"pair_index")?,degree,side).map_err(|e|e.to_string())?;
            json!({"pair_ref":f.pair.operator_ref(),"degree":f.degree.as_str(),"projection_side":f.expansion_side.map(|s|s.as_str()),"coordinates":f.coordinates.into_iter().map(coordinate).collect::<Vec<_>>()})
        },
        "cross"=>match v["kind"].as_str(){
            Some("same-position")=>cross(canonical_cross_pass_d1(position(v,"position")?)),
            Some("transform")=>cross(canonical_cross_pass_d2(D2CrossPassKind::Transform,position(v,"position")?)),
            Some("require")=>cross(canonical_cross_pass_d2(D2CrossPassKind::Require,position(v,"position")?)),
            Some("complete")=>cross(canonical_cross_pass_d2(D2CrossPassKind::Complete,position(v,"position")?)),
            Some("invariance")=>cross(canonical_cross_pass_d3(family(v)?)),_=>return Err("unknown cross kind".into())},
        "kernel"=>{
            let operator=match v["operator"].as_str(){Some("conjugate-address")=>QlOperator::ConjugateAddress,Some("complement-address")=>QlOperator::ComplementAddress,Some("classify-four-plus-two")=>QlOperator::ClassifyFourPlusTwo,_=>return Err("unsupported owner operator".into())};
            let depth=v["depth"].as_u64().and_then(|n|u32::try_from(n).ok()).ok_or("depth requires u32")?;
            let a=QlAddress::sixfold(position(v,"position")?.value(),face(v)?,depth).map_err(|e|e.to_string())?;
            let r=apply_operator(operator,a);
            json!({"value":r.value.to_string(),"provenance":{"schema_version":r.provenance.schema_version,"kernel_version":r.provenance.kernel_version,"operation":r.provenance.operation,"input":r.provenance.input,"output":r.provenance.output}})
        },
        "harmonic-snapshot"=>{
            let basis=match v["basis"].as_str(){Some("chromatic")=>MusicalBasis::Chromatic,Some("fifths")=>MusicalBasis::Fifths,_=>return Err("unknown musical basis".into())};
            let d=derive_pre_m_music(basis);
            let pairs=[RelationFamily::A,RelationFamily::B,RelationFamily::C].into_iter().map(|f|(f.as_str().to_owned(),json!(pair_interval_deltas(basis,f,QlFace::Direct)))).collect::<serde_json::Map<_,_>>();
            let crosses=[("same-position",CrossOperator::SamePosition),("transform",CrossOperator::Transform),("require",CrossOperator::Require),("complete",CrossOperator::Complete)].into_iter().map(|(k,op)|(k.to_owned(),json!(cross_interval_deltas(basis,op)))).collect::<serde_json::Map<_,_>>();
            json!({"basis":v["basis"],"direct_helix":d.direct_helix,"conjugate_helix":d.conjugate_helix,"lens_anchor_pitches":d.lens_anchors.iter().map(|a|a.pitch).collect::<Vec<_>>(),"mode_tonic_instances":d.mode_tonic_landscape.len(),"pair_intervals":pairs,"cross_intervals":crosses})
        },_=>return Err("unsupported owner operation".into()),
    };
    Ok(
        json!({"schema":SCHEMA,"owner_repository":"EpiLogos/QL-MEF","owner_revision":OWNER_REVISION,"operation":operation,"result":result}),
    )
}
fn main() {
    let result = (|| {
        let mut bytes = Vec::new();
        io::stdin()
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| "input read failed")?;
        if bytes.len() > 1_048_576 {
            return Err("request exceeds input bound".into());
        }
        let v = serde_json::from_slice(&bytes).map_err(|_| "invalid JSON request")?;
        invoke(&v)
    })();
    match result {
        Ok(v) => println!("{v}"),
        Err(message) => {
            println!(
                "{}",
                json!({"schema":SCHEMA,"error":message,"owner_revision":OWNER_REVISION})
            );
            std::process::exit(1);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vocabulary_comes_from_owner() {
        let r = invoke(&json!({"operation":"vocabulary"})).unwrap();
        assert_eq!(r["result"]["positions"].as_array().unwrap().len(), 6);
        assert_eq!(r["result"]["families"][0]["pairs"][0]["left"], 0);
    }
    #[test]
    fn ambiguity_survives_owner_relation() {
        let r = invoke(&json!({"operation":"classify-relation","from":2,"to":3})).unwrap();
        assert_eq!(r["result"].as_array().unwrap().len(), 2);
        assert!(invoke(&json!({"operation":"classify-relation","from":6,"to":3})).is_err());
    }
    #[test]
    fn owner_refuses_wrong_degree() {
        assert!(invoke(
            &json!({"operation":"modulation","family":"A","pair_index":0,"degree":"D2"})
        )
        .is_err());
        assert!(invoke(&json!({"operation":"modulation","family":"A","pair_index":0,"degree":"D1","projection_side":"left"})).is_err());
    }
    #[test]
    fn complete_grammar_and_music() {
        for f in ["A", "B", "C"] {
            for p in 0..3 {
                for d in ["D1", "D3"] {
                    assert!(invoke(
                        &json!({"operation":"modulation","family":f,"pair_index":p,"degree":d})
                    )
                    .is_ok());
                }
                for s in ["left", "right"] {
                    assert!(invoke(&json!({"operation":"modulation","family":f,"pair_index":p,"degree":"D2","projection_side":s})).is_ok());
                }
            }
        }
        for b in ["chromatic", "fifths"] {
            assert!(invoke(&json!({"operation":"harmonic-snapshot","basis":b})).is_ok());
        }
        for n in 0..6 {
            for k in ["same-position", "transform", "require", "complete"] {
                assert!(invoke(&json!({"operation":"cross","kind":k,"position":n})).is_ok());
            }
        }
    }
}
