// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const { Digest, checkCredential, KeyType, publish, VerifiableCredential, VerificationMethod, KeyCollection } = require('../node/identity_wasm')
const { createIdentity } = require('./create_did');
const { logExplorerUrl } = require('./explorer_util')

/*
    This example shows how to sign/revoke verifiable credentials on scale.
    Instead of revoking the entire verification method, a single key can be revoked from a MerkleKeyCollection.
    This MerkleKeyCollection can be created as a collection of a power of 2 amount of keys.
    Every key should be used once by the issuer for signing a verifiable credential.
    When the verifiable credential must be revoked, the issuer revokes the index of the revoked key.

    @param {{network: string, node: string}} clientConfig
*/
async function merkleKey(clientConfig) {
    //Creates new identities (See "create_did" example)
    const alice = await createIdentity(clientConfig);
    const issuer = await createIdentity(clientConfig);

    //Add a Merkle Key Collection Verification Method with 8 keys (Must be a power of 2)
    const keys = new KeyCollection(KeyType.Ed25519, 8);
    const method = VerificationMethod.createMerkleKey(Digest.Sha256, issuer.doc.id, keys, "key-collection")

    // Add to the DID Document as a general-purpose verification method
    issuer.doc.insertMethod(method, "VerificationMethod");
    issuer.doc.previousMessageId = issuer.messageId;
    issuer.doc.sign(issuer.key);

    //Publish the Identity to the IOTA Network and log the results, this may take a few seconds to complete Proof-of-Work.
    const nextMessageId = await publish(issuer.doc.toJSON(), clientConfig);
    logExplorerUrl("Identity Update:", clientConfig.network, nextMessageId);

    // Prepare a credential subject indicating the degree earned by Alice
    let credentialSubject = {
        id: alice.doc.id.toString(),
        name: "Alice",
        degreeName: "Bachelor of Science and Arts",
        degreeType: "BachelorDegree",
        GPA: "4.0"
    };

    // Create an unsigned `UniversityDegree` credential for Alice
    const unsignedVc = VerifiableCredential.extend({
        id: "http://example.edu/credentials/3732",
        type: "UniversityDegreeCredential",
        issuer: issuer.doc.id.toString(),
        credentialSubject,
    });

    // Sign the credential with Issuer's Merkle Key Collection method, with key index 0
    const signedVc = issuer.doc.signCredential(unsignedVc, {
        method: method.id.toString(),
        public: keys.public(0),
        secret: keys.secret(0),
        proof: keys.merkleProof(Digest.Sha256, 0)
    });

    //Check the verifiable credential
    const result = await checkCredential(signedVc.toString(), clientConfig);
    console.log(`VC verification result: ${result.verified}`);

    // The Issuer would like to revoke the credential (and therefore revokes key 0)
    issuer.doc.revokeMerkleKey(method.id.toString(), 0);
    issuer.doc.previousMessageId = nextMessageId;
    const revokeMessageId = await publish(issuer.doc.toJSON(), clientConfig);
    logExplorerUrl("Identity Update:", clientConfig.network, revokeMessageId);

    //Check the verifiable credential
    const newResult = await checkCredential(signedVc.toString(), clientConfig);
    console.log(`VC verification result: ${newResult.verified}`);
}

exports.merkleKey = merkleKey;
