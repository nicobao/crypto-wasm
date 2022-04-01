use serde_wasm_bindgen::*;
use wasm_bindgen::prelude::*;

use crate::accumulator::{
    deserialize_params, deserialize_public_key, MembershipPrk, MembershipWit, NonMembershipPrk,
    NonMembershipWit,
};
use crate::bbs_plus::{BBSPlusPkG2, SigG1, SigParamsG1};
use crate::common::VerifyResponse;
use crate::utils::{
    fr_from_uint8_array, g1_affine_from_uint8_array, g2_affine_from_uint8_array, get_seeded_rng,
    js_array_to_fr_vec, js_array_to_g1_affine_vec, js_array_to_g2_affine_vec,
    msgs_bytes_map_to_fr_btreemap, set_panic_hook,
};
use crate::{Fr, G1Affine, G2Affine};
use ark_bls12_381::Bls12_381;
use ark_ec::{AffineCurve, PairingEngine};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::collections::BTreeSet;
use blake2::Blake2b;
use js_sys::Uint8Array;
use proof_system::prelude::{EqualWitnesses, MetaStatement, MetaStatements, Statement};
use proof_system::proof;
use proof_system::statement;
use proof_system::witness;

use crate::bound_check::BoundCheckSnarkPk;
use crate::saver::{ChunkedCommGens, EncGens, SaverEk, SaverSnarkPk};

pub(crate) type PoKBBSSigStmt = statement::PoKBBSSignatureG1<Bls12_381>;
pub(crate) type AccumMemStmt = statement::AccumulatorMembership<Bls12_381>;
pub(crate) type AccumNonMemStmt = statement::AccumulatorNonMembership<Bls12_381>;
pub(crate) type PedCommG1Stmt =
    statement::PedersenCommitment<<Bls12_381 as PairingEngine>::G1Affine>;
pub(crate) type PedCommG2Stmt =
    statement::PedersenCommitment<<Bls12_381 as PairingEngine>::G2Affine>;
pub(crate) type SaverStmt = statement::Saver<Bls12_381>;
pub(crate) type BoundCheckLegoStmt = statement::BoundCheckLegoGroth16<Bls12_381>;
pub type Witness = witness::Witness<Bls12_381>;
pub type Witnesses = witness::Witnesses<Bls12_381>;
pub(crate) type PoKBBSSigWit = witness::PoKBBSSignatureG1<Bls12_381>;
pub(crate) type AccumMemWit = witness::Membership<Bls12_381>;
pub(crate) type AccumNonMemWit = witness::NonMembership<Bls12_381>;
pub(crate) type ProofSpec<G> = proof_system::proof_spec::ProofSpec<Bls12_381, G>;
pub(crate) type Proof<G> = proof::Proof<Bls12_381, G, Blake2b>;
pub(crate) type ProofG1 = proof::Proof<Bls12_381, G1Affine, Blake2b>;
pub(crate) type StatementProofG1 = proof_system::prelude::StatementProof<Bls12_381, G1Affine>;

#[wasm_bindgen(js_name = generatePoKBBSSignatureStatement)]
pub fn generate_pok_bbs_sig_statement(
    params: JsValue,
    public_key: JsValue,
    revealed_msgs: js_sys::Map,
    encode_messages: bool,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let params: SigParamsG1 = serde_wasm_bindgen::from_value(params)?;
    let pk: BBSPlusPkG2 = serde_wasm_bindgen::from_value(public_key)?;
    let msgs = msgs_bytes_map_to_fr_btreemap(&revealed_msgs, encode_messages)?;
    let statement = PoKBBSSigStmt::new_as_statement::<G1Affine>(params, pk, msgs);
    Ok(obj_to_uint8array_unchecked!(&statement, "PokBBSStatement"))
}

#[wasm_bindgen(js_name = generateAccumulatorMembershipStatement)]
pub fn generate_accumulator_membership_statement(
    params: js_sys::Uint8Array,
    public_key: js_sys::Uint8Array,
    proving_key: js_sys::Uint8Array,
    accumulated: js_sys::Uint8Array,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let accumulated = g1_affine_from_uint8_array(accumulated)?;
    let pk = deserialize_public_key(public_key)?;
    let params = deserialize_params(params)?;
    let prk = obj_from_uint8array!(MembershipPrk, proving_key, "MembershipPrk");
    let statement = AccumMemStmt::new_as_statement::<G1Affine>(params, pk, prk, accumulated);
    Ok(obj_to_uint8array_unchecked!(
        &statement,
        "AccumMemStatement"
    ))
}

#[wasm_bindgen(js_name = generateAccumulatorNonMembershipStatement)]
pub fn generate_accumulator_non_membership_statement(
    params: js_sys::Uint8Array,
    public_key: js_sys::Uint8Array,
    proving_key: js_sys::Uint8Array,
    accumulated: js_sys::Uint8Array,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let accumulated = g1_affine_from_uint8_array(accumulated)?;
    let pk = deserialize_public_key(public_key)?;
    let params = deserialize_params(params)?;
    let prk = obj_from_uint8array!(NonMembershipPrk, proving_key, "NonMembershipPrk");
    let statement = AccumNonMemStmt::new_as_statement::<G1Affine>(params, pk, prk, accumulated);
    Ok(obj_to_uint8array_unchecked!(
        &statement,
        "AccumNonMemStatement"
    ))
}

#[wasm_bindgen(js_name = generatePedersenCommitmentG1Statement)]
pub fn generate_pedersen_commitment_g1_statement(
    bases: js_sys::Array,
    commitment: js_sys::Uint8Array,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let bases = js_array_to_g1_affine_vec(&bases)?;
    let commitment = g1_affine_from_uint8_array(commitment)?;
    let statement = PedCommG1Stmt::new_as_statement::<Bls12_381>(bases, commitment);
    Ok(obj_to_uint8array_unchecked!(&statement, "PedCommG1Stmt"))
}

#[wasm_bindgen(js_name = generatePedersenCommitmentG2Statement)]
pub fn generate_pedersen_commitment_g2_statement(
    bases: js_sys::Array,
    commitment: js_sys::Uint8Array,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let bases = js_array_to_g2_affine_vec(&bases)?;
    let commitment = g2_affine_from_uint8_array(commitment)?;
    let statement = PedCommG2Stmt::new_as_statement::<Bls12_381>(bases, commitment);
    Ok(obj_to_uint8array_unchecked!(&statement, "PedCommG2Stmt"))
}

#[wasm_bindgen(js_name = generateWitnessEqualityMetaStatement)]
pub fn generate_witness_equality_meta_statement(equality: js_sys::Set) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let mut set = BTreeSet::new();
    for wr in equality.values() {
        let wr = wr.unwrap();
        let arr_2 = js_sys::Array::from(&wr);
        if arr_2.length() != 2 {
            return Err(JsValue::from("Each equality should be a 2 element array"));
        }
        let i: u32 = serde_wasm_bindgen::from_value(arr_2.get(0)).unwrap();
        let j: u32 = serde_wasm_bindgen::from_value(arr_2.get(1)).unwrap();
        set.insert((i as usize, j as usize));
    }
    serde_wasm_bindgen::to_value(&MetaStatement::WitnessEquality(EqualWitnesses(set)))
        .map_err(|e| JsValue::from(e))
}

#[wasm_bindgen(js_name = generatePoKBBSSignatureWitness)]
pub fn generate_pok_bbs_sig_witness(
    signature: js_sys::Uint8Array,
    unrevealed_msgs: js_sys::Map,
    encode_messages: bool,
) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let signature = obj_from_uint8array!(SigG1, signature);
    let msgs = msgs_bytes_map_to_fr_btreemap(&unrevealed_msgs, encode_messages)?;
    let witness = PoKBBSSigWit::new_as_witness(signature, msgs);
    serde_wasm_bindgen::to_value(&witness).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen(js_name = generateAccumulatorMembershipWitness)]
pub fn generate_accumulator_membership_witness(
    element: js_sys::Uint8Array,
    accum_witness: JsValue,
) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let element = fr_from_uint8_array(element)?;
    let accum_witness: MembershipWit = serde_wasm_bindgen::from_value(accum_witness)?;
    let witness = AccumMemWit::new_as_witness(element, accum_witness);
    serde_wasm_bindgen::to_value(&witness).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen(js_name = generateAccumulatorNonMembershipWitness)]
pub fn generate_accumulator_non_membership_witness(
    element: js_sys::Uint8Array,
    accum_witness: JsValue,
) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let element = fr_from_uint8_array(element)?;
    let accum_witness: NonMembershipWit = serde_wasm_bindgen::from_value(accum_witness)?;
    let witness = AccumNonMemWit::new_as_witness(element, accum_witness);
    serde_wasm_bindgen::to_value(&witness).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen(js_name = generatePedersenCommitmentWitness)]
pub fn generate_pedersen_commitment_witness(elements: js_sys::Array) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let elements = js_array_to_fr_vec(&elements)?;
    let witness = Witness::PedersenCommitment(elements);
    serde_wasm_bindgen::to_value(&witness).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen(js_name = generateProofSpecG1)]
pub fn generate_proof_spec_g1(
    statements: js_sys::Array,
    meta_statements: js_sys::Array,
    context: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    gen_proof_spec::<<Bls12_381 as PairingEngine>::G1Affine>(statements, meta_statements, context)
}

#[wasm_bindgen(js_name = generateProofSpecG2)]
pub fn generate_proof_spec_g2(
    statements: js_sys::Array,
    meta_statements: js_sys::Array,
    context: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    gen_proof_spec::<<Bls12_381 as PairingEngine>::G2Affine>(statements, meta_statements, context)
}

#[wasm_bindgen(js_name = generateCompositeProofG1)]
pub fn generate_composite_proof_g1(
    proof_spec: js_sys::Uint8Array,
    witnesses: js_sys::Array,
    nonce: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    gen_proof::<<Bls12_381 as PairingEngine>::G1Affine>(proof_spec, witnesses, nonce)
}

#[wasm_bindgen(js_name = generateCompositeProofG2)]
pub fn generate_composite_proof_g2(
    proof_spec: js_sys::Uint8Array,
    witnesses: js_sys::Array,
    nonce: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    gen_proof::<<Bls12_381 as PairingEngine>::G2Affine>(proof_spec, witnesses, nonce)
}

#[wasm_bindgen(js_name = generateCompositeProofG1WithDeconstructedProofSpec)]
pub fn generate_composite_proof_g1_with_deconstructed_proof_spec(
    statements: js_sys::Array,
    meta_statements: js_sys::Array,
    witnesses: js_sys::Array,
    context: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let (statements, meta_statements) =
        parse_statements_and_meta_statements(statements, meta_statements)?;
    let proof_spec = ProofSpec::<<Bls12_381 as PairingEngine>::G1Affine>::new_with_statements_and_meta_statements(statements, meta_statements, context);
    gen_proof_given_proof_spec_obj::<<Bls12_381 as PairingEngine>::G1Affine>(
        proof_spec, witnesses, nonce,
    )
}

#[wasm_bindgen(js_name = verifyCompositeProofG1)]
pub fn verify_composite_proof_g1(
    proof: js_sys::Uint8Array,
    proof_spec: js_sys::Uint8Array,
    nonce: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    set_panic_hook();
    verify_proof::<<Bls12_381 as PairingEngine>::G1Affine>(proof_spec, proof, nonce)
}

#[wasm_bindgen(js_name = verifyCompositeProofG2)]
pub fn verify_composite_proof_g2(
    proof: js_sys::Uint8Array,
    proof_spec: js_sys::Uint8Array,
    nonce: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    set_panic_hook();
    verify_proof::<<Bls12_381 as PairingEngine>::G2Affine>(proof_spec, proof, nonce)
}

#[wasm_bindgen(js_name = verifyCompositeProofG1WithDeconstructedProofSpec)]
pub fn verify_composite_proof_g1_with_deconstructed_proof_spec(
    proof: js_sys::Uint8Array,
    statements: js_sys::Array,
    meta_statements: js_sys::Array,
    context: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    let (statements, meta_statements) =
        parse_statements_and_meta_statements(statements, meta_statements)?;
    let proof_spec = ProofSpec::<<Bls12_381 as PairingEngine>::G1Affine>::new_with_statements_and_meta_statements(statements, meta_statements, context);
    verify_proof_given_proof_spec_obj::<<Bls12_381 as PairingEngine>::G1Affine>(
        proof_spec, proof, nonce,
    )
}

#[wasm_bindgen(js_name = generateSaverStatement)]
pub fn generate_saver_statement(
    chunk_bit_size: u8,
    enc_gens: js_sys::Uint8Array,
    chunked_comm_gens: js_sys::Uint8Array,
    encryption_key: js_sys::Uint8Array,
    snark_pk: js_sys::Uint8Array,
    uncompressed_public_params: bool,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let (enc_gens, chunked_comm_gens, ek, snark_pk) = if uncompressed_public_params {
        (
            obj_from_uint8array_unchecked!(EncGens, enc_gens, "EncryptionGenerators"),
            obj_from_uint8array_unchecked!(
                ChunkedCommGens,
                chunked_comm_gens,
                "ChunkedCommitmentGenerators"
            ),
            obj_from_uint8array_unchecked!(SaverEk, encryption_key, "SaverEk"),
            obj_from_uint8array_unchecked!(SaverSnarkPk, snark_pk, "SaverSnarkPk"),
        )
    } else {
        (
            obj_from_uint8array!(EncGens, enc_gens, "EncryptionGenerators"),
            obj_from_uint8array!(
                ChunkedCommGens,
                chunked_comm_gens,
                "ChunkedCommitmentGenerators"
            ),
            obj_from_uint8array!(SaverEk, encryption_key, "SaverEk"),
            obj_from_uint8array!(SaverSnarkPk, snark_pk, "SaverSnarkPk"),
        )
    };
    let statement = SaverStmt::new_as_statement::<G1Affine>(
        chunk_bit_size,
        enc_gens,
        chunked_comm_gens,
        ek,
        snark_pk,
    )
    .map_err(|e| {
        JsValue::from(&format!(
            "Creating statement for SAVER returned error: {:?}",
            e
        ))
    })?;
    Ok(obj_to_uint8array_unchecked!(&statement, "SaverStatement"))
}

#[wasm_bindgen(js_name = generateSaverWitness)]
pub fn generate_saver_witness(message: js_sys::Uint8Array) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let message = fr_from_uint8_array(message)?;
    let witness = Witness::Saver(message);
    serde_wasm_bindgen::to_value(&witness).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen(js_name = saverGetCiphertextFromProof)]
pub fn saver_get_ciphertext_from_proof(
    proof: js_sys::Uint8Array,
    statement_index: usize,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let proof = obj_from_uint8array!(ProofG1, proof);
    let statement_proof = proof
        .statement_proof(statement_index)
        .map_err(|_| JsValue::from(&format!("Did not find StatementProof at the given index")))?;
    if let StatementProofG1::Saver(s) = statement_proof {
        Ok(obj_to_uint8array!(&s.ciphertext, "SaverCiphertext"))
    } else {
        Err(JsValue::from(&format!("StatementProof wasn't for Saver")))
    }
}

#[wasm_bindgen(js_name = generateBoundCheckLegoStatement)]
pub fn generate_bound_check_lego_statement(
    min: Uint8Array,
    max: Uint8Array,
    snark_pk: js_sys::Uint8Array,
    uncompressed_public_params: bool,
) -> Result<js_sys::Uint8Array, JsValue> {
    set_panic_hook();
    let min = fr_from_uint8_array(min)?;
    let max = fr_from_uint8_array(max)?;
    let snark_pk = if uncompressed_public_params {
        obj_from_uint8array_unchecked!(BoundCheckSnarkPk, snark_pk, "BoundCheckSnarkPk")
    } else {
        obj_from_uint8array!(BoundCheckSnarkPk, snark_pk, "BoundCheckSnarkPk")
    };
    let statement =
        BoundCheckLegoStmt::new_as_statement::<G1Affine>(min, max, snark_pk).map_err(|e| {
            JsValue::from(&format!(
                "Creating statement for SAVER returned error: {:?}",
                e
            ))
        })?;
    Ok(obj_to_uint8array_unchecked!(
        &statement,
        "BoundCheckLegoStmt"
    ))
}

#[wasm_bindgen(js_name = generateBoundCheckWitness)]
pub fn generate_bound_check_witness(message: js_sys::Uint8Array) -> Result<JsValue, JsValue> {
    set_panic_hook();
    let message = fr_from_uint8_array(message)?;
    let witness = Witness::BoundCheckLegoGroth16(message);
    serde_wasm_bindgen::to_value(&witness).map_err(|e| JsValue::from(e))
}

fn gen_proof_spec<G: AffineCurve>(
    statements: js_sys::Array,
    meta_statements: js_sys::Array,
    context: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let (stmts, meta_stmts) = parse_statements_and_meta_statements(statements, meta_statements)?;
    let proof_spec =
        ProofSpec::<G>::new_with_statements_and_meta_statements(stmts, meta_stmts, context);
    Ok(obj_to_uint8array_unchecked!(&proof_spec, "ProofSpec"))
}

fn gen_proof<G: AffineCurve<ScalarField = Fr>>(
    proof_spec: js_sys::Uint8Array,
    witnesses: js_sys::Array,
    nonce: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let proof_spec = obj_from_uint8array_unchecked!(ProofSpec::<G>, proof_spec, "ProofSpec");
    gen_proof_given_proof_spec_obj::<G>(proof_spec, witnesses, nonce)
}

fn verify_proof<G: AffineCurve<ScalarField = Fr>>(
    proof_spec: js_sys::Uint8Array,
    proof: js_sys::Uint8Array,
    nonce: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    let proof_spec = obj_from_uint8array_unchecked!(ProofSpec::<G>, proof_spec, "ProofSpec");
    verify_proof_given_proof_spec_obj::<G>(proof_spec, proof, nonce)
}

fn gen_proof_given_proof_spec_obj<G: AffineCurve<ScalarField = Fr>>(
    proof_spec: ProofSpec<G>,
    witnesses: js_sys::Array,
    nonce: Option<Vec<u8>>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let mut wits: Witnesses = witness::Witnesses::new();
    for w in witnesses.values() {
        let wit: Witness = serde_wasm_bindgen::from_value(w.unwrap())?;
        wits.add(wit);
    }
    let mut rng = get_seeded_rng();
    let proof = Proof::<G>::new(&mut rng, proof_spec, wits, nonce)
        .map_err(|e| JsValue::from(&format!("Generating proof returned error: {:?}", e)))?;
    Ok(obj_to_uint8array!(&proof, "Proof"))
}

fn verify_proof_given_proof_spec_obj<G: AffineCurve<ScalarField = Fr>>(
    proof_spec: ProofSpec<G>,
    proof: js_sys::Uint8Array,
    nonce: Option<Vec<u8>>,
) -> Result<JsValue, JsValue> {
    let proof = obj_from_uint8array!(Proof<G>, proof);
    match proof.verify(proof_spec, nonce) {
        Ok(_) => Ok(serde_wasm_bindgen::to_value(&VerifyResponse {
            verified: true,
            error: None,
        })
        .unwrap()),
        Err(e) => Ok(serde_wasm_bindgen::to_value(&VerifyResponse {
            verified: false,
            error: Some(format!("Verifying proof returned error {:?}", e)),
        })
        .unwrap()),
    }
}

fn parse_statements_and_meta_statements<G: AffineCurve>(
    statements: js_sys::Array,
    meta_statements: js_sys::Array,
) -> Result<(statement::Statements<Bls12_381, G>, MetaStatements), JsValue> {
    let mut meta_stmts = MetaStatements::new();
    for ms in meta_statements.values() {
        let meta_stmt: MetaStatement = serde_wasm_bindgen::from_value(ms.unwrap())?;
        meta_stmts.add(meta_stmt);
    }
    let mut stmts: statement::Statements<Bls12_381, G> = statement::Statements::new();
    for s in statements.values() {
        let s = js_sys::Uint8Array::new(&s.unwrap());
        let stmt =
            obj_from_uint8array_unchecked!(statement::Statement<Bls12_381, G>, &s, "Statement");
        stmts.add(stmt);
    }
    Ok((stmts, meta_stmts))
}
