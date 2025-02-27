#![cfg(target_arch = "wasm32")]
extern crate wasm_bindgen_test;

use web_sys::console;

use ark_bls12_381::Bls12_381;
use ark_ec::pairing::Pairing;
use ark_serialize::CanonicalDeserialize;
use ark_std::collections::BTreeSet;
use dock_crypto_wasm::{
    accumulator::{
        accumulator_derive_membership_proving_key_from_non_membership_key,
        generate_non_membership_proving_key, positive_accumulator_add,
        positive_accumulator_get_accumulated, positive_accumulator_initialize,
        positive_accumulator_membership_witness, universal_accumulator_add,
        universal_accumulator_compute_d, universal_accumulator_get_accumulated,
        universal_accumulator_membership_witness, universal_accumulator_non_membership_witness,
    },
    bbs::*,
    bbs_plus::{
        bbs_plus_blind_sign_g1, bbs_plus_commit_to_message_in_g1,
        bbs_plus_get_bases_for_commitment_g1, bbs_plus_sign_g1, bbs_plus_unblind_sig_g1,
        bbs_plus_verify_g1,
    },
    common::{
        encode_message_for_signing, encode_messages_for_signing, field_element_as_bytes,
        field_element_from_number, generate_field_element_from_bytes,
        generate_random_field_element, generate_random_g1_element, generate_random_g2_element,
        pedersen_commitment_g1, pedersen_commitment_g2, VerifyResponse,
    },
    composite_proof_system::{
        generate_accumulator_membership_witness, generate_accumulator_non_membership_witness,
        generate_composite_proof_g1, generate_composite_proof_g2,
        generate_pedersen_commitment_witness, generate_pok_bbs_plus_sig_witness,
        generate_pok_bbs_sig_witness, generate_proof_spec_g1, generate_proof_spec_g2,
        setup_params::{
            generate_setup_param_for_vb_accumulator_mem_proving_key,
            generate_setup_param_for_vb_accumulator_non_mem_proving_key,
            generate_setup_param_for_vb_accumulator_params,
            generate_setup_param_for_vb_accumulator_public_key,
        },
        verify_composite_proof_g1, verify_composite_proof_g2, Witness,
    },
    utils::{
        encode_messages_as_js_map_to_fr_btreemap, fr_from_jsvalue,
        js_array_of_bytearrays_from_vector_of_bytevectors, random_bytes,
    },
};
use proof_system::statement;
mod common;
use common::{
    accum_params_and_keys, bbs_params_and_keys, gen_msgs, get_revealed_unrevealed,
    get_universal_accum, get_witness_equality_statement,
};
use dock_crypto_wasm::composite_proof_system::statement::{
    generate_accumulator_membership_statement,
    generate_accumulator_membership_statement_from_param_refs,
    generate_accumulator_non_membership_statement,
    generate_accumulator_non_membership_statement_from_param_refs,
    generate_pedersen_commitment_g1_statement, generate_pedersen_commitment_g2_statement,
    generate_pok_bbs_plus_sig_statement, generate_pok_bbs_sig_statement,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn test_bbs_statement(stmt_j: js_sys::Uint8Array, revealed_msgs: js_sys::Map) {
    let s = js_sys::Uint8Array::new(&stmt_j);
    let serz = s.to_vec();
    let stmt: statement::Statement<Bls12_381, <Bls12_381 as Pairing>::G1Affine> =
        CanonicalDeserialize::deserialize_uncompressed(&serz[..]).unwrap();
    match stmt {
        statement::Statement::PoKBBSSignature23G1(s) => {
            assert_eq!(s.revealed_messages.len() as u32, revealed_msgs.size());
            for (i, m) in s.revealed_messages.iter() {
                assert_eq!(
                    *m,
                    fr_from_jsvalue(revealed_msgs.get(&JsValue::from(*i as u32))).unwrap()
                );
            }
        }
        _ => assert!(false),
    }
}

fn test_bbs_witness(wit_j: JsValue, unrevealed_msgs: js_sys::Map) {
    let wit: Witness = serde_wasm_bindgen::from_value(wit_j).unwrap();
    match wit {
        Witness::PoKBBSSignature23G1(s) => {
            assert_eq!(s.unrevealed_messages.len() as u32, unrevealed_msgs.size());
            for (i, m) in s.unrevealed_messages.iter() {
                assert_eq!(
                    *m,
                    fr_from_jsvalue(unrevealed_msgs.get(&JsValue::from(*i as u32))).unwrap()
                );
            }
        }
        _ => assert!(false),
    }
}

fn test_bbs_plus_statement(stmt_j: js_sys::Uint8Array, revealed_msgs: js_sys::Map) {
    let s = js_sys::Uint8Array::new(&stmt_j);
    let serz = s.to_vec();
    let stmt: statement::Statement<Bls12_381, <Bls12_381 as Pairing>::G1Affine> =
        CanonicalDeserialize::deserialize_uncompressed(&serz[..]).unwrap();
    match stmt {
        statement::Statement::PoKBBSSignatureG1(s) => {
            assert_eq!(s.revealed_messages.len() as u32, revealed_msgs.size());
            for (i, m) in s.revealed_messages.iter() {
                assert_eq!(
                    *m,
                    fr_from_jsvalue(revealed_msgs.get(&JsValue::from(*i as u32))).unwrap()
                );
            }
        }
        _ => assert!(false),
    }
}

fn test_bbs_plus_witness(wit_j: JsValue, unrevealed_msgs: js_sys::Map) {
    let wit: Witness = serde_wasm_bindgen::from_value(wit_j).unwrap();
    match wit {
        Witness::PoKBBSSignatureG1(s) => {
            assert_eq!(s.unrevealed_messages.len() as u32, unrevealed_msgs.size());
            for (i, m) in s.unrevealed_messages.iter() {
                assert_eq!(
                    *m,
                    fr_from_jsvalue(unrevealed_msgs.get(&JsValue::from(*i as u32))).unwrap()
                );
            }
        }
        _ => assert!(false),
    }
}

#[allow(non_snake_case)]
#[wasm_bindgen_test]
pub fn three_bbs_plus_sigs_and_msg_equality() {
    let msg_count_1 = 5;
    let (params_1, sk_1, pk_1) = bbs_params_and_keys(msg_count_1 as u32);
    let msgs_1 = gen_msgs(msg_count_1);

    let msg_count_2 = 6;
    let (params_2, sk_2, pk_2) = bbs_params_and_keys(msg_count_2 as u32);
    let mut msgs_2 = gen_msgs(msg_count_2);

    let msg_count_3 = 7;
    let (params_3, sk_3, pk_3) = bbs_params_and_keys(msg_count_3 as u32);
    let mut msgs_3 = gen_msgs(msg_count_3);

    // Message at index 2 in msgs_1 is equal to index 3 in msgs_2
    msgs_2[3] = msgs_1[2].clone();
    // Message at index 2 in msgs_1 is equal to index 3 in msgs_3
    msgs_3[3] = msgs_1[2].clone();

    let msgs_1_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_1).unwrap();
    let msgs_2_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_2).unwrap();
    let msgs_3_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_3).unwrap();
    let sig_1 = bbs_plus_sign_g1(msgs_1_as_array, sk_1, params_1.clone(), true).unwrap();
    let sig_2 = bbs_plus_sign_g1(msgs_2_as_array, sk_2, params_2.clone(), true).unwrap();
    let sig_3 = bbs_plus_sign_g1(msgs_3_as_array, sk_3, params_3.clone(), true).unwrap();

    // Prepare revealed messages for the proof of knowledge of 1st signature
    let mut revealed_indices_1 = BTreeSet::new();
    revealed_indices_1.insert(0);
    let (revealed_msgs_1, unrevealed_msgs_1) =
        get_revealed_unrevealed(&msgs_1, &revealed_indices_1);

    // Prepare revealed messages for the proof of knowledge of 2nd signature
    let mut revealed_indices_2 = BTreeSet::new();
    revealed_indices_2.insert(1);
    let (revealed_msgs_2, unrevealed_msgs_2) =
        get_revealed_unrevealed(&msgs_2, &revealed_indices_2);
    let (revealed_msgs_3, unrevealed_msgs_3) = get_revealed_unrevealed(&msgs_3, &BTreeSet::new());

    // Create statements
    let stmt_1 =
        generate_pok_bbs_plus_sig_statement(params_1, pk_1, revealed_msgs_1, true).unwrap();
    let stmt_2 =
        generate_pok_bbs_plus_sig_statement(params_2, pk_2, revealed_msgs_2, true).unwrap();
    let stmt_3 =
        generate_pok_bbs_plus_sig_statement(params_3, pk_3, revealed_msgs_3, true).unwrap();

    let meta_statements = js_sys::Array::new();

    // Create equality meta-statement, statement 0's 2nd index = statement 1st's 3rd index = statement 2nd's 3rd index
    let meta_statement = get_witness_equality_statement(vec![(0, 2), (1, 3), (2, 3)]);
    meta_statements.push(&meta_statement);

    let statements = js_sys::Array::new();
    statements.push(&stmt_1);
    statements.push(&stmt_2);
    statements.push(&stmt_3);

    let context = Some("test-context".as_bytes().to_vec());

    let proof_spec =
        generate_proof_spec_g1(statements, meta_statements, js_sys::Array::new(), context).unwrap();

    let witness_1 = generate_pok_bbs_plus_sig_witness(sig_1, unrevealed_msgs_1, true).unwrap();
    let witness_2 = generate_pok_bbs_plus_sig_witness(sig_2, unrevealed_msgs_2, true).unwrap();
    let witness_3 = generate_pok_bbs_plus_sig_witness(sig_3, unrevealed_msgs_3, true).unwrap();

    let witnesses = js_sys::Array::new();
    witnesses.push(&witness_1);
    witnesses.push(&witness_2);
    witnesses.push(&witness_3);

    let nonce = Some("test-nonce".as_bytes().to_vec());

    console::time_with_label("proof gen");
    let proof = generate_composite_proof_g1(proof_spec.clone(), witnesses, nonce.clone()).unwrap();
    console::time_end_with_label("proof gen");

    console::time_with_label("proof ver");
    let result = verify_composite_proof_g1(proof, proof_spec, nonce).unwrap();
    console::time_end_with_label("proof ver");
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();
}

#[allow(non_snake_case)]
#[wasm_bindgen_test]
pub fn bbs_plus_sig_and_accumulator() {
    fn run(use_setup_params: bool) {
        let member_1 = field_element_as_bytes(
            field_element_from_number(js_sys::Number::from(5)).unwrap(),
            true,
        )
        .unwrap();
        let member_2 = field_element_as_bytes(
            field_element_from_number(js_sys::Number::from(10)).unwrap(),
            true,
        )
        .unwrap();
        let member_3 = field_element_as_bytes(
            generate_field_element_from_bytes("user_1232".as_bytes().to_vec()),
            true,
        )
        .unwrap();

        let msg_count_1 = 5;
        let (params_1, sk_1, pk_1) = bbs_params_and_keys(msg_count_1);
        let mut msgs_1 = vec![];
        for _ in 0..msg_count_1 - 2 {
            let m = random_bytes();
            let bytes = encode_message_for_signing(m).unwrap();
            msgs_1.push(bytes.to_vec());
        }

        msgs_1.push(member_1.to_vec());
        msgs_1.push(member_2.to_vec());

        let msgs_1_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_1).unwrap();
        let sig_1 = bbs_plus_sign_g1(msgs_1_as_array, sk_1, params_1.clone(), false).unwrap();

        let msg_count_2 = 6;
        let (params_2, sk_2, pk_2) = bbs_params_and_keys(msg_count_2);
        let mut msgs_2 = vec![];
        for _ in 0..msg_count_2 as usize - 2 {
            let m = random_bytes();
            let bytes = encode_message_for_signing(m).unwrap();
            msgs_2.push(bytes.to_vec());
        }

        // Message at index 2 in msgs_1 is equal to index 3 in msgs_2
        msgs_2[3] = msgs_1[2].clone();
        assert_eq!(msgs_2[3], msgs_1[2]);
        // msgs_1 has member_1 at index 3 and msgs_2 has member_1 at index 4
        msgs_2.push(member_1.to_vec());
        msgs_2.push(member_3.to_vec());

        assert_eq!(msgs_2[4], msgs_1[3]);

        assert_eq!(msgs_1[3], member_1.to_vec());
        assert_eq!(msgs_1[4], member_2.to_vec());
        assert_eq!(msgs_2[4], member_1.to_vec());
        assert_eq!(msgs_2[5], member_3.to_vec());

        let msgs_2_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_2).unwrap();
        let sig_2 = bbs_plus_sign_g1(msgs_2_as_array, sk_2, params_2.clone(), false).unwrap();

        // Prepare revealed messages for the proof of knowledge of 1st signature
        let mut revealed_indices_1 = BTreeSet::new();
        revealed_indices_1.insert(0);
        let (revealed_msgs_1, unrevealed_msgs_1) =
            get_revealed_unrevealed(&msgs_1, &revealed_indices_1);

        // Prepare revealed messages for the proof of knowledge of 2nd signature
        let mut revealed_indices_2 = BTreeSet::new();
        revealed_indices_2.insert(1);
        let (revealed_msgs_2, unrevealed_msgs_2) =
            get_revealed_unrevealed(&msgs_2, &revealed_indices_2);

        let (accum_params, accum_sk, accum_pk) = accum_params_and_keys();
        let non_mem_prk = generate_non_membership_proving_key(None).unwrap();
        let mem_prk =
            accumulator_derive_membership_proving_key_from_non_membership_key(non_mem_prk.clone())
                .unwrap();

        let mut pos_accumulator = positive_accumulator_initialize(accum_params.clone()).unwrap();

        let max_size = 10;
        let mut uni_accumulator =
            get_universal_accum(accum_sk.clone(), accum_params.clone(), max_size);

        let non_member = generate_random_field_element(None).unwrap();

        pos_accumulator =
            positive_accumulator_add(pos_accumulator, member_1.clone(), accum_sk.clone()).unwrap();
        pos_accumulator =
            positive_accumulator_add(pos_accumulator, member_2.clone(), accum_sk.clone()).unwrap();
        pos_accumulator =
            positive_accumulator_add(pos_accumulator, member_3.clone(), accum_sk.clone()).unwrap();
        let pos_witness_1 = positive_accumulator_membership_witness(
            pos_accumulator.clone(),
            member_1.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let pos_witness_2 = positive_accumulator_membership_witness(
            pos_accumulator.clone(),
            member_2.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let pos_witness_3 = positive_accumulator_membership_witness(
            pos_accumulator.clone(),
            member_3.clone(),
            accum_sk.clone(),
        )
        .unwrap();

        uni_accumulator =
            universal_accumulator_add(uni_accumulator, member_1.clone(), accum_sk.clone()).unwrap();
        uni_accumulator =
            universal_accumulator_add(uni_accumulator, member_2.clone(), accum_sk.clone()).unwrap();
        uni_accumulator =
            universal_accumulator_add(uni_accumulator, member_3.clone(), accum_sk.clone()).unwrap();
        let uni_witness_1 = universal_accumulator_membership_witness(
            uni_accumulator.clone(),
            member_1.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let uni_witness_2 = universal_accumulator_membership_witness(
            uni_accumulator.clone(),
            member_2.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let uni_witness_3 = universal_accumulator_membership_witness(
            uni_accumulator.clone(),
            member_3.clone(),
            accum_sk.clone(),
        )
        .unwrap();

        let members = js_sys::Array::new();
        members.push(&member_1);
        members.push(&member_2);
        members.push(&member_3);

        let d = universal_accumulator_compute_d(non_member.clone(), members).unwrap();
        let nm_witness = universal_accumulator_non_membership_witness(
            uni_accumulator.clone(),
            d,
            non_member.clone(),
            accum_sk,
            accum_params.clone(),
        )
        .unwrap();

        let pos_accumulated = positive_accumulator_get_accumulated(pos_accumulator).unwrap();
        let uni_accumulated = universal_accumulator_get_accumulated(uni_accumulator).unwrap();

        let setup_params = js_sys::Array::new();
        // Create statements
        let (stmt_1, stmt_2, stmt_3, stmt_4, stmt_5, stmt_6, stmt_7, stmt_8, stmt_9) =
            if use_setup_params {
                setup_params
                    .push(&generate_setup_param_for_vb_accumulator_params(accum_params).unwrap());
                setup_params
                    .push(&generate_setup_param_for_vb_accumulator_public_key(accum_pk).unwrap());
                setup_params.push(
                    &generate_setup_param_for_vb_accumulator_mem_proving_key(mem_prk).unwrap(),
                );
                setup_params.push(
                    &generate_setup_param_for_vb_accumulator_non_mem_proving_key(non_mem_prk)
                        .unwrap(),
                );

                let stmt_1 = generate_pok_bbs_plus_sig_statement(
                    params_1,
                    pk_1,
                    revealed_msgs_1.clone(),
                    false,
                )
                .unwrap();
                let stmt_2 = generate_pok_bbs_plus_sig_statement(
                    params_2,
                    pk_2,
                    revealed_msgs_2.clone(),
                    false,
                )
                .unwrap();
                // Membership of member_1 in positive accumulator
                let stmt_3 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in positive accumulator
                let stmt_4 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in positive accumulator
                let stmt_5 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    pos_accumulated,
                )
                .unwrap();
                // Membership of member_1 in universal accumulator
                let stmt_6 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in universal accumulator
                let stmt_7 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in universal accumulator
                let stmt_8 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    uni_accumulated.clone(),
                )
                .unwrap();
                let stmt_9 = generate_accumulator_non_membership_statement_from_param_refs(
                    0,
                    1,
                    3,
                    uni_accumulated,
                )
                .unwrap();

                (
                    stmt_1, stmt_2, stmt_3, stmt_4, stmt_5, stmt_6, stmt_7, stmt_8, stmt_9,
                )
            } else {
                let stmt_1 = generate_pok_bbs_plus_sig_statement(
                    params_1,
                    pk_1,
                    revealed_msgs_1.clone(),
                    false,
                )
                .unwrap();
                let stmt_2 = generate_pok_bbs_plus_sig_statement(
                    params_2,
                    pk_2,
                    revealed_msgs_2.clone(),
                    false,
                )
                .unwrap();
                // Membership of member_1 in positive accumulator
                let stmt_3 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in positive accumulator
                let stmt_4 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in positive accumulator
                let stmt_5 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    pos_accumulated,
                )
                .unwrap();
                // Membership of member_1 in universal accumulator
                let stmt_6 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in universal accumulator
                let stmt_7 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in universal accumulator
                let stmt_8 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk,
                    uni_accumulated.clone(),
                )
                .unwrap();
                let stmt_9 = generate_accumulator_non_membership_statement(
                    accum_params,
                    accum_pk,
                    non_mem_prk,
                    uni_accumulated,
                )
                .unwrap();
                (
                    stmt_1, stmt_2, stmt_3, stmt_4, stmt_5, stmt_6, stmt_7, stmt_8, stmt_9,
                )
            };

        let meta_statements = js_sys::Array::new();

        // statement 0's 2nd index = statement 1st's 3rd index
        let meta_statement = get_witness_equality_statement(vec![(0, 2), (1, 3)]);
        meta_statements.push(&meta_statement);

        // statement 0's 3nd index = statement 1st's 4th index = statement 2nd's 0th index = statement 5th's 0th index
        let meta_statement = get_witness_equality_statement(vec![(0, 3), (1, 4), (2, 0), (5, 0)]);
        meta_statements.push(&meta_statement);

        let meta_statement = get_witness_equality_statement(vec![(2, 0), (5, 0)]);
        meta_statements.push(&meta_statement);

        let meta_statement = get_witness_equality_statement(vec![(3, 0), (6, 0)]);
        meta_statements.push(&meta_statement);

        let meta_statement = get_witness_equality_statement(vec![(4, 0), (7, 0)]);
        meta_statements.push(&meta_statement);

        let statements = js_sys::Array::new();
        statements.push(&stmt_1);
        statements.push(&stmt_2);
        statements.push(&stmt_3);
        statements.push(&stmt_4);
        statements.push(&stmt_5);
        statements.push(&stmt_6);
        statements.push(&stmt_7);
        statements.push(&stmt_8);
        statements.push(&stmt_9);

        meta_statements.push(&meta_statement);

        let context = Some("test-context".as_bytes().to_vec());

        let proof_spec =
            generate_proof_spec_g1(statements, meta_statements, setup_params, context).unwrap();

        let witness_1 =
            generate_pok_bbs_plus_sig_witness(sig_1, unrevealed_msgs_1.clone(), false).unwrap();
        let witness_2 =
            generate_pok_bbs_plus_sig_witness(sig_2, unrevealed_msgs_2.clone(), false).unwrap();
        let witness_3 =
            generate_accumulator_membership_witness(member_1.clone(), pos_witness_1).unwrap();
        let witness_4 =
            generate_accumulator_membership_witness(member_2.clone(), pos_witness_2).unwrap();
        let witness_5 =
            generate_accumulator_membership_witness(member_3.clone(), pos_witness_3).unwrap();
        let witness_6 = generate_accumulator_membership_witness(member_1, uni_witness_1).unwrap();
        let witness_7 = generate_accumulator_membership_witness(member_2, uni_witness_2).unwrap();
        let witness_8 = generate_accumulator_membership_witness(member_3, uni_witness_3).unwrap();
        let witness_9 =
            generate_accumulator_non_membership_witness(non_member, nm_witness).unwrap();

        let witnesses = js_sys::Array::new();
        witnesses.push(&witness_1);
        witnesses.push(&witness_2);
        witnesses.push(&witness_3);
        witnesses.push(&witness_4);
        witnesses.push(&witness_5);
        witnesses.push(&witness_6);
        witnesses.push(&witness_7);
        witnesses.push(&witness_8);
        witnesses.push(&witness_9);

        let msgs = encode_messages_as_js_map_to_fr_btreemap(&revealed_msgs_1, false).unwrap();
        assert_eq!(msgs.len(), 1);

        test_bbs_plus_statement(stmt_1, revealed_msgs_1);
        test_bbs_plus_statement(stmt_2, revealed_msgs_2);
        test_bbs_plus_witness(witness_1, unrevealed_msgs_1);
        test_bbs_plus_witness(witness_2, unrevealed_msgs_2);

        let nonce = Some("test-nonce".as_bytes().to_vec());

        let proof =
            generate_composite_proof_g1(proof_spec.clone(), witnesses, nonce.clone()).unwrap();

        let result = verify_composite_proof_g1(proof, proof_spec, nonce).unwrap();
        let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
        r.validate();
    }
    run(true);
    run(false);
}

#[allow(non_snake_case)]
#[wasm_bindgen_test]
pub fn bbs_sig_and_accumulator() {
    fn run(use_setup_params: bool) {
        let member_1 = field_element_as_bytes(
            field_element_from_number(js_sys::Number::from(5)).unwrap(),
            true,
        )
        .unwrap();
        let member_2 = field_element_as_bytes(
            field_element_from_number(js_sys::Number::from(10)).unwrap(),
            true,
        )
        .unwrap();
        let member_3 = field_element_as_bytes(
            generate_field_element_from_bytes("user_1232".as_bytes().to_vec()),
            true,
        )
        .unwrap();

        let msg_count_1 = 5;
        let (params_1, sk_1, pk_1) = bbs_params_and_keys(msg_count_1);
        let mut msgs_1 = vec![];
        for _ in 0..msg_count_1 - 2 {
            let m = random_bytes();
            let bytes = encode_message_for_signing(m).unwrap();
            msgs_1.push(bytes.to_vec());
        }

        msgs_1.push(member_1.to_vec());
        msgs_1.push(member_2.to_vec());

        let msgs_1_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_1).unwrap();
        let sig_1 = bbs_sign(msgs_1_as_array, sk_1, params_1.clone(), false).unwrap();

        let msg_count_2 = 6;
        let (params_2, sk_2, pk_2) = bbs_params_and_keys(msg_count_2);
        let mut msgs_2 = vec![];
        for _ in 0..msg_count_2 as usize - 2 {
            let m = random_bytes();
            let bytes = encode_message_for_signing(m).unwrap();
            msgs_2.push(bytes.to_vec());
        }

        // Message at index 2 in msgs_1 is equal to index 3 in msgs_2
        msgs_2[3] = msgs_1[2].clone();
        assert_eq!(msgs_2[3], msgs_1[2]);
        // msgs_1 has member_1 at index 3 and msgs_2 has member_1 at index 4
        msgs_2.push(member_1.to_vec());
        msgs_2.push(member_3.to_vec());

        assert_eq!(msgs_2[4], msgs_1[3]);

        assert_eq!(msgs_1[3], member_1.to_vec());
        assert_eq!(msgs_1[4], member_2.to_vec());
        assert_eq!(msgs_2[4], member_1.to_vec());
        assert_eq!(msgs_2[5], member_3.to_vec());

        let msgs_2_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_2).unwrap();
        let sig_2 = bbs_sign(msgs_2_as_array, sk_2, params_2.clone(), false).unwrap();

        // Prepare revealed messages for the proof of knowledge of 1st signature
        let mut revealed_indices_1 = BTreeSet::new();
        revealed_indices_1.insert(0);
        let (revealed_msgs_1, unrevealed_msgs_1) =
            get_revealed_unrevealed(&msgs_1, &revealed_indices_1);

        // Prepare revealed messages for the proof of knowledge of 2nd signature
        let mut revealed_indices_2 = BTreeSet::new();
        revealed_indices_2.insert(1);
        let (revealed_msgs_2, unrevealed_msgs_2) =
            get_revealed_unrevealed(&msgs_2, &revealed_indices_2);

        let (accum_params, accum_sk, accum_pk) = accum_params_and_keys();
        let non_mem_prk = generate_non_membership_proving_key(None).unwrap();
        let mem_prk =
            accumulator_derive_membership_proving_key_from_non_membership_key(non_mem_prk.clone())
                .unwrap();

        let mut pos_accumulator = positive_accumulator_initialize(accum_params.clone()).unwrap();

        let max_size = 10;
        let mut uni_accumulator =
            get_universal_accum(accum_sk.clone(), accum_params.clone(), max_size);

        let non_member = generate_random_field_element(None).unwrap();

        pos_accumulator =
            positive_accumulator_add(pos_accumulator, member_1.clone(), accum_sk.clone()).unwrap();
        pos_accumulator =
            positive_accumulator_add(pos_accumulator, member_2.clone(), accum_sk.clone()).unwrap();
        pos_accumulator =
            positive_accumulator_add(pos_accumulator, member_3.clone(), accum_sk.clone()).unwrap();
        let pos_witness_1 = positive_accumulator_membership_witness(
            pos_accumulator.clone(),
            member_1.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let pos_witness_2 = positive_accumulator_membership_witness(
            pos_accumulator.clone(),
            member_2.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let pos_witness_3 = positive_accumulator_membership_witness(
            pos_accumulator.clone(),
            member_3.clone(),
            accum_sk.clone(),
        )
        .unwrap();

        uni_accumulator =
            universal_accumulator_add(uni_accumulator, member_1.clone(), accum_sk.clone()).unwrap();
        uni_accumulator =
            universal_accumulator_add(uni_accumulator, member_2.clone(), accum_sk.clone()).unwrap();
        uni_accumulator =
            universal_accumulator_add(uni_accumulator, member_3.clone(), accum_sk.clone()).unwrap();
        let uni_witness_1 = universal_accumulator_membership_witness(
            uni_accumulator.clone(),
            member_1.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let uni_witness_2 = universal_accumulator_membership_witness(
            uni_accumulator.clone(),
            member_2.clone(),
            accum_sk.clone(),
        )
        .unwrap();
        let uni_witness_3 = universal_accumulator_membership_witness(
            uni_accumulator.clone(),
            member_3.clone(),
            accum_sk.clone(),
        )
        .unwrap();

        let members = js_sys::Array::new();
        members.push(&member_1);
        members.push(&member_2);
        members.push(&member_3);

        let d = universal_accumulator_compute_d(non_member.clone(), members).unwrap();
        let nm_witness = universal_accumulator_non_membership_witness(
            uni_accumulator.clone(),
            d,
            non_member.clone(),
            accum_sk,
            accum_params.clone(),
        )
        .unwrap();

        let pos_accumulated = positive_accumulator_get_accumulated(pos_accumulator).unwrap();
        let uni_accumulated = universal_accumulator_get_accumulated(uni_accumulator).unwrap();

        let setup_params = js_sys::Array::new();
        // Create statements
        let (stmt_1, stmt_2, stmt_3, stmt_4, stmt_5, stmt_6, stmt_7, stmt_8, stmt_9) =
            if use_setup_params {
                setup_params
                    .push(&generate_setup_param_for_vb_accumulator_params(accum_params).unwrap());
                setup_params
                    .push(&generate_setup_param_for_vb_accumulator_public_key(accum_pk).unwrap());
                setup_params.push(
                    &generate_setup_param_for_vb_accumulator_mem_proving_key(mem_prk).unwrap(),
                );
                setup_params.push(
                    &generate_setup_param_for_vb_accumulator_non_mem_proving_key(non_mem_prk)
                        .unwrap(),
                );

                let stmt_1 =
                    generate_pok_bbs_sig_statement(params_1, pk_1, revealed_msgs_1.clone(), false)
                        .unwrap();
                let stmt_2 =
                    generate_pok_bbs_sig_statement(params_2, pk_2, revealed_msgs_2.clone(), false)
                        .unwrap();
                // Membership of member_1 in positive accumulator
                let stmt_3 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in positive accumulator
                let stmt_4 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in positive accumulator
                let stmt_5 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    pos_accumulated,
                )
                .unwrap();
                // Membership of member_1 in universal accumulator
                let stmt_6 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in universal accumulator
                let stmt_7 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in universal accumulator
                let stmt_8 = generate_accumulator_membership_statement_from_param_refs(
                    0,
                    1,
                    2,
                    uni_accumulated.clone(),
                )
                .unwrap();
                let stmt_9 = generate_accumulator_non_membership_statement_from_param_refs(
                    0,
                    1,
                    3,
                    uni_accumulated,
                )
                .unwrap();

                (
                    stmt_1, stmt_2, stmt_3, stmt_4, stmt_5, stmt_6, stmt_7, stmt_8, stmt_9,
                )
            } else {
                let stmt_1 =
                    generate_pok_bbs_sig_statement(params_1, pk_1, revealed_msgs_1.clone(), false)
                        .unwrap();
                let stmt_2 =
                    generate_pok_bbs_sig_statement(params_2, pk_2, revealed_msgs_2.clone(), false)
                        .unwrap();
                // Membership of member_1 in positive accumulator
                let stmt_3 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in positive accumulator
                let stmt_4 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    pos_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in positive accumulator
                let stmt_5 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    pos_accumulated,
                )
                .unwrap();
                // Membership of member_1 in universal accumulator
                let stmt_6 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_2 in universal accumulator
                let stmt_7 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk.clone(),
                    uni_accumulated.clone(),
                )
                .unwrap();
                // Membership of member_3 in universal accumulator
                let stmt_8 = generate_accumulator_membership_statement(
                    accum_params.clone(),
                    accum_pk.clone(),
                    mem_prk,
                    uni_accumulated.clone(),
                )
                .unwrap();
                let stmt_9 = generate_accumulator_non_membership_statement(
                    accum_params,
                    accum_pk,
                    non_mem_prk,
                    uni_accumulated,
                )
                .unwrap();
                (
                    stmt_1, stmt_2, stmt_3, stmt_4, stmt_5, stmt_6, stmt_7, stmt_8, stmt_9,
                )
            };

        let meta_statements = js_sys::Array::new();

        // statement 0's 2nd index = statement 1st's 3rd index
        let meta_statement = get_witness_equality_statement(vec![(0, 2), (1, 3)]);
        meta_statements.push(&meta_statement);

        // statement 0's 3nd index = statement 1st's 4th index = statement 2nd's 0th index = statement 5th's 0th index
        let meta_statement = get_witness_equality_statement(vec![(0, 3), (1, 4), (2, 0), (5, 0)]);
        meta_statements.push(&meta_statement);

        let meta_statement = get_witness_equality_statement(vec![(2, 0), (5, 0)]);
        meta_statements.push(&meta_statement);

        let meta_statement = get_witness_equality_statement(vec![(3, 0), (6, 0)]);
        meta_statements.push(&meta_statement);

        let meta_statement = get_witness_equality_statement(vec![(4, 0), (7, 0)]);
        meta_statements.push(&meta_statement);

        let statements = js_sys::Array::new();
        statements.push(&stmt_1);
        statements.push(&stmt_2);
        statements.push(&stmt_3);
        statements.push(&stmt_4);
        statements.push(&stmt_5);
        statements.push(&stmt_6);
        statements.push(&stmt_7);
        statements.push(&stmt_8);
        statements.push(&stmt_9);

        meta_statements.push(&meta_statement);

        let context = Some("test-context".as_bytes().to_vec());

        let proof_spec =
            generate_proof_spec_g1(statements, meta_statements, setup_params, context).unwrap();

        let witness_1 =
            generate_pok_bbs_sig_witness(sig_1, unrevealed_msgs_1.clone(), false).unwrap();
        let witness_2 =
            generate_pok_bbs_sig_witness(sig_2, unrevealed_msgs_2.clone(), false).unwrap();
        let witness_3 =
            generate_accumulator_membership_witness(member_1.clone(), pos_witness_1).unwrap();
        let witness_4 =
            generate_accumulator_membership_witness(member_2.clone(), pos_witness_2).unwrap();
        let witness_5 =
            generate_accumulator_membership_witness(member_3.clone(), pos_witness_3).unwrap();
        let witness_6 = generate_accumulator_membership_witness(member_1, uni_witness_1).unwrap();
        let witness_7 = generate_accumulator_membership_witness(member_2, uni_witness_2).unwrap();
        let witness_8 = generate_accumulator_membership_witness(member_3, uni_witness_3).unwrap();
        let witness_9 =
            generate_accumulator_non_membership_witness(non_member, nm_witness).unwrap();

        let witnesses = js_sys::Array::new();
        witnesses.push(&witness_1);
        witnesses.push(&witness_2);
        witnesses.push(&witness_3);
        witnesses.push(&witness_4);
        witnesses.push(&witness_5);
        witnesses.push(&witness_6);
        witnesses.push(&witness_7);
        witnesses.push(&witness_8);
        witnesses.push(&witness_9);

        let msgs = encode_messages_as_js_map_to_fr_btreemap(&revealed_msgs_1, false).unwrap();
        assert_eq!(msgs.len(), 1);

        test_bbs_statement(stmt_1, revealed_msgs_1);
        test_bbs_statement(stmt_2, revealed_msgs_2);
        test_bbs_witness(witness_1, unrevealed_msgs_1);
        test_bbs_witness(witness_2, unrevealed_msgs_2);

        let nonce = Some("test-nonce".as_bytes().to_vec());

        let proof =
            generate_composite_proof_g1(proof_spec.clone(), witnesses, nonce.clone()).unwrap();

        let result = verify_composite_proof_g1(proof, proof_spec, nonce).unwrap();
        let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
        r.validate();
    }
    run(true);
    run(false);
}

#[allow(non_snake_case)]
#[wasm_bindgen_test]
pub fn request_blind_bbs_sig() {
    let msg_count_1 = 5;
    let (params_1, sk_1, pk_1) = bbs_params_and_keys(msg_count_1);
    let msgs_1 = gen_msgs(msg_count_1);

    let msg_count_2 = 6;
    let (params_2, sk_2, pk_2) = bbs_params_and_keys(msg_count_2);
    let mut msgs_2 = gen_msgs(msg_count_2);

    // One message is equal
    msgs_2[5] = msgs_1[4].clone();

    let msgs_1_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_1).unwrap();
    let sig_1 = bbs_sign(msgs_1_as_array, sk_1, params_1.clone(), true).unwrap();

    let msgs_2_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_2).unwrap();

    let mut revealed_indices = BTreeSet::new();
    revealed_indices.insert(0);
    let (revealed_msgs_1, unrevealed_msgs_1) = get_revealed_unrevealed(&msgs_1, &revealed_indices);

    let committed_indices = vec![0, 1, 5];
    let indices_to_commit = js_sys::Array::new();
    let msgs_to_commit = js_sys::Map::new();
    let msgs_to_not_commit = js_sys::Map::new();
    for i in 0..msg_count_2 as usize {
        if committed_indices.contains(&i) {
            indices_to_commit.push(&JsValue::from(i as u32));
            msgs_to_commit.set(
                &JsValue::from(i as u32),
                &serde_wasm_bindgen::to_value(&msgs_2[i]).unwrap(),
            );
        } else {
            msgs_to_not_commit.set(
                &JsValue::from(i as u32),
                &serde_wasm_bindgen::to_value(&msgs_2[i]).unwrap(),
            );
        }
    }

    let commitment = bbs_commit_to_message(msgs_to_commit, params_2.clone(), true).unwrap();

    let statements = js_sys::Array::new();
    let stmt_1 = generate_pok_bbs_sig_statement(params_1, pk_1, revealed_msgs_1, true).unwrap();
    statements.push(&stmt_1);

    let bases = bbs_get_bases_for_commitment(params_2.clone(), indices_to_commit.clone()).unwrap();
    let stmt_2 = generate_pedersen_commitment_g1_statement(bases, commitment.clone()).unwrap();
    statements.push(&stmt_2);

    let context = Some("test-context".as_bytes().to_vec());

    let proof_spec = generate_proof_spec_g1(
        statements,
        Default::default(),
        js_sys::Array::new(),
        context,
    )
    .unwrap();

    let witness_1 = generate_pok_bbs_sig_witness(sig_1, unrevealed_msgs_1, true).unwrap();

    let wits =
        encode_messages_for_signing(msgs_2_as_array.clone(), Some(indices_to_commit)).unwrap();
    let witness_2 = generate_pedersen_commitment_witness(wits).unwrap();

    let witnesses = js_sys::Array::new();
    witnesses.push(&witness_1);
    witnesses.push(&witness_2);

    let nonce = Some("test-nonce".as_bytes().to_vec());
    let proof = generate_composite_proof_g1(proof_spec.clone(), witnesses, nonce.clone()).unwrap();
    let result = verify_composite_proof_g1(proof, proof_spec, nonce).unwrap();
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();

    let sig_2 =
        bbs_blind_sign(commitment, msgs_to_not_commit, sk_2, params_2.clone(), true).unwrap();

    let result = bbs_verify(msgs_2_as_array, sig_2, pk_2, params_2, true).unwrap();
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();
}

#[allow(non_snake_case)]
#[wasm_bindgen_test]
pub fn request_blind_bbs_plus_sig() {
    let msg_count_1 = 5;
    let (params_1, sk_1, pk_1) = bbs_params_and_keys(msg_count_1);
    let msgs_1 = gen_msgs(msg_count_1);

    let msg_count_2 = 6;
    let (params_2, sk_2, pk_2) = bbs_params_and_keys(msg_count_2);
    let mut msgs_2 = gen_msgs(msg_count_2);

    // One message is equal
    msgs_2[5] = msgs_1[4].clone();

    let msgs_1_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_1).unwrap();
    let sig_1 = bbs_plus_sign_g1(msgs_1_as_array, sk_1, params_1.clone(), true).unwrap();

    let msgs_2_as_array = js_array_of_bytearrays_from_vector_of_bytevectors(&msgs_2).unwrap();

    let mut revealed_indices = BTreeSet::new();
    revealed_indices.insert(0);
    let (revealed_msgs_1, unrevealed_msgs_1) = get_revealed_unrevealed(&msgs_1, &revealed_indices);

    let committed_indices = vec![0, 1, 5];
    let indices_to_commit = js_sys::Array::new();
    let msgs_to_commit = js_sys::Map::new();
    let msgs_to_not_commit = js_sys::Map::new();
    for i in 0..msg_count_2 as usize {
        if committed_indices.contains(&i) {
            indices_to_commit.push(&JsValue::from(i as u32));
            msgs_to_commit.set(
                &JsValue::from(i as u32),
                &serde_wasm_bindgen::to_value(&msgs_2[i]).unwrap(),
            );
        } else {
            msgs_to_not_commit.set(
                &JsValue::from(i as u32),
                &serde_wasm_bindgen::to_value(&msgs_2[i]).unwrap(),
            );
        }
    }
    let blinding = generate_random_field_element(None).unwrap();

    let commitment =
        bbs_plus_commit_to_message_in_g1(msgs_to_commit, blinding.clone(), params_2.clone(), true)
            .unwrap();

    let statements = js_sys::Array::new();
    let stmt_1 =
        generate_pok_bbs_plus_sig_statement(params_1, pk_1, revealed_msgs_1, true).unwrap();
    statements.push(&stmt_1);

    let bases =
        bbs_plus_get_bases_for_commitment_g1(params_2.clone(), indices_to_commit.clone()).unwrap();
    let stmt_2 = generate_pedersen_commitment_g1_statement(bases, commitment.clone()).unwrap();
    statements.push(&stmt_2);

    let meta_statements = js_sys::Array::new();
    let meta_statement = get_witness_equality_statement(vec![(0, 4), (1, 3)]);
    meta_statements.push(&meta_statement);

    let context = Some("test-context".as_bytes().to_vec());

    let proof_spec =
        generate_proof_spec_g1(statements, meta_statements, js_sys::Array::new(), context).unwrap();

    let witness_1 = generate_pok_bbs_plus_sig_witness(sig_1, unrevealed_msgs_1, true).unwrap();

    let wits =
        encode_messages_for_signing(msgs_2_as_array.clone(), Some(indices_to_commit)).unwrap();
    wits.unshift(&blinding);
    let witness_2 = generate_pedersen_commitment_witness(wits).unwrap();

    let witnesses = js_sys::Array::new();
    witnesses.push(&witness_1);
    witnesses.push(&witness_2);

    let nonce = Some("test-nonce".as_bytes().to_vec());
    let proof = generate_composite_proof_g1(proof_spec.clone(), witnesses, nonce.clone()).unwrap();
    let result = verify_composite_proof_g1(proof, proof_spec, nonce).unwrap();
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();

    let blinded_sig =
        bbs_plus_blind_sign_g1(commitment, msgs_to_not_commit, sk_2, params_2.clone(), true)
            .unwrap();
    let sig_2 = bbs_plus_unblind_sig_g1(blinded_sig, blinding).unwrap();

    let result = bbs_plus_verify_g1(msgs_2_as_array, sig_2, pk_2, params_2, true).unwrap();
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();
}

#[allow(non_snake_case)]
#[wasm_bindgen_test]
pub fn pedersen_commitment_opening_equality() {
    let m_1 = generate_random_field_element(None).unwrap();
    let m_2 = generate_random_field_element(None).unwrap();
    let m_3 = generate_random_field_element(None).unwrap();

    let msgs_1 = js_sys::Array::new();
    msgs_1.push(&m_1);
    msgs_1.push(&m_2);

    let msgs_2 = js_sys::Array::new();
    msgs_2.push(&m_1);
    msgs_2.push(&m_2);
    msgs_2.push(&m_3);

    let bases_1 = js_sys::Array::new();
    bases_1.push(&generate_random_g1_element(None).unwrap());
    bases_1.push(&generate_random_g1_element(None).unwrap());

    let comm_1 = pedersen_commitment_g1(bases_1.clone(), msgs_1.clone()).unwrap();

    let bases_2 = js_sys::Array::new();
    bases_2.push(&generate_random_g1_element(None).unwrap());
    bases_2.push(&generate_random_g1_element(None).unwrap());
    bases_2.push(&generate_random_g1_element(None).unwrap());

    let comm_2 = pedersen_commitment_g1(bases_2.clone(), msgs_2.clone()).unwrap();

    let statements = js_sys::Array::new();
    let stmt_1 = generate_pedersen_commitment_g1_statement(bases_1, comm_1).unwrap();
    let stmt_2 = generate_pedersen_commitment_g1_statement(bases_2, comm_2).unwrap();
    statements.push(&stmt_1);
    statements.push(&stmt_2);

    let meta_statements = js_sys::Array::new();
    meta_statements.push(&get_witness_equality_statement(vec![(0, 0), (1, 0)]));
    meta_statements.push(&get_witness_equality_statement(vec![(0, 1), (1, 1)]));

    let proof_spec =
        generate_proof_spec_g1(statements, meta_statements, js_sys::Array::new(), None).unwrap();

    let witnesses = js_sys::Array::new();
    witnesses.push(&generate_pedersen_commitment_witness(msgs_1.clone()).unwrap());
    witnesses.push(&generate_pedersen_commitment_witness(msgs_2.clone()).unwrap());

    let proof = generate_composite_proof_g1(proof_spec.clone(), witnesses, None).unwrap();
    let result = verify_composite_proof_g1(proof, proof_spec, None).unwrap();
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();

    let bases_1 = js_sys::Array::new();
    bases_1.push(&generate_random_g2_element(None).unwrap());
    bases_1.push(&generate_random_g2_element(None).unwrap());

    let comm_1 = pedersen_commitment_g2(bases_1.clone(), msgs_1.clone()).unwrap();

    let bases_2 = js_sys::Array::new();
    bases_2.push(&generate_random_g2_element(None).unwrap());
    bases_2.push(&generate_random_g2_element(None).unwrap());
    bases_2.push(&generate_random_g2_element(None).unwrap());

    let comm_2 = pedersen_commitment_g2(bases_2.clone(), msgs_2.clone()).unwrap();

    let statements = js_sys::Array::new();
    let stmt_1 = generate_pedersen_commitment_g2_statement(bases_1, comm_1).unwrap();
    let stmt_2 = generate_pedersen_commitment_g2_statement(bases_2, comm_2).unwrap();
    statements.push(&stmt_1);
    statements.push(&stmt_2);

    let meta_statements = js_sys::Array::new();
    meta_statements.push(&get_witness_equality_statement(vec![(0, 0), (1, 0)]));
    meta_statements.push(&get_witness_equality_statement(vec![(0, 1), (1, 1)]));

    let proof_spec =
        generate_proof_spec_g2(statements, meta_statements, js_sys::Array::new(), None).unwrap();

    let witnesses = js_sys::Array::new();
    witnesses.push(&generate_pedersen_commitment_witness(msgs_1).unwrap());
    witnesses.push(&generate_pedersen_commitment_witness(msgs_2).unwrap());

    let proof = generate_composite_proof_g2(proof_spec.clone(), witnesses, None).unwrap();
    let result = verify_composite_proof_g2(proof, proof_spec, None).unwrap();
    let r: VerifyResponse = serde_wasm_bindgen::from_value(result).unwrap();
    r.validate();
}
