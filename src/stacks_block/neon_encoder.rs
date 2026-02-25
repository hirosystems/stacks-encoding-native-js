use neon::prelude::*;

use crate::hex::encode_hex;
use crate::neon_util::NeonJsSerialize;

use super::deserialize::{
    BitVec, NakamotoBlock, NakamotoBlockHeader, StacksBlock, StacksBlockHeader,
    StacksWorkScore,
};

impl NeonJsSerialize for NakamotoBlock {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        // Serialize header
        let header_obj = cx.empty_object();
        self.header.neon_js_serialize(cx, &header_obj, &())?;
        obj.set(cx, "header", header_obj)?;

        // Serialize transactions
        let txs_array = JsArray::new(cx, self.txs.len());
        for (i, tx) in self.txs.iter().enumerate() {
            let tx_obj = cx.empty_object();
            tx.neon_js_serialize(cx, &tx_obj, &())?;
            txs_array.set(cx, i as u32, tx_obj)?;
        }
        obj.set(cx, "txs", txs_array)?;

        Ok(())
    }
}

impl NeonJsSerialize for NakamotoBlockHeader {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let version = cx.number(self.version);
        obj.set(cx, "version", version)?;

        let chain_length = cx.string(self.chain_length.to_string());
        obj.set(cx, "chain_length", chain_length)?;

        let burn_spent = cx.string(self.burn_spent.to_string());
        obj.set(cx, "burn_spent", burn_spent)?;

        let consensus_hash = cx.string(encode_hex(&self.consensus_hash.0));
        obj.set(cx, "consensus_hash", consensus_hash)?;

        let parent_block_id = cx.string(encode_hex(&self.parent_block_id.0));
        obj.set(cx, "parent_block_id", parent_block_id)?;

        let tx_merkle_root = cx.string(encode_hex(&self.tx_merkle_root.0));
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let state_index_root = cx.string(encode_hex(&self.state_index_root.0));
        obj.set(cx, "state_index_root", state_index_root)?;

        let timestamp = cx.string(self.timestamp.to_string());
        obj.set(cx, "timestamp", timestamp)?;

        let miner_signature = cx.string(encode_hex(&self.miner_signature.0));
        obj.set(cx, "miner_signature", miner_signature)?;

        // Signer signatures array
        let signer_sigs_array = JsArray::new(cx, self.signer_signature.len());
        for (i, sig) in self.signer_signature.iter().enumerate() {
            let sig_hex = cx.string(encode_hex(&sig.0));
            signer_sigs_array.set(cx, i as u32, sig_hex)?;
        }
        obj.set(cx, "signer_signature", signer_sigs_array)?;

        // PoX treatment bitvec
        let pox_treatment_obj = cx.empty_object();
        self.pox_treatment.neon_js_serialize(cx, &pox_treatment_obj, &())?;
        obj.set(cx, "pox_treatment", pox_treatment_obj)?;

        // Computed values
        let block_hash = cx.string(encode_hex(&self.block_hash()));
        obj.set(cx, "block_hash", block_hash)?;

        let block_id = cx.string(encode_hex(&self.block_id()));
        obj.set(cx, "index_block_hash", block_id)?;

        Ok(())
    }
}

impl NeonJsSerialize for BitVec {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let len = cx.number(self.len);
        obj.set(cx, "len", len)?;

        let data = cx.string(encode_hex(&self.data));
        obj.set(cx, "data", data)?;

        // Also provide a human-readable array of booleans
        let bits_array = JsArray::new(cx, self.len as usize);
        for i in 0..self.len {
            let bit_val = cx.boolean(self.get(i).unwrap_or(false));
            bits_array.set(cx, i as u32, bit_val)?;
        }
        obj.set(cx, "bits", bits_array)?;

        Ok(())
    }
}

impl NeonJsSerialize for StacksBlock {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        // Serialize header
        let header_obj = cx.empty_object();
        self.header.neon_js_serialize(cx, &header_obj, &())?;
        obj.set(cx, "header", header_obj)?;

        // Serialize transactions
        let txs_array = JsArray::new(cx, self.txs.len());
        for (i, tx) in self.txs.iter().enumerate() {
            let tx_obj = cx.empty_object();
            tx.neon_js_serialize(cx, &tx_obj, &())?;
            txs_array.set(cx, i as u32, tx_obj)?;
        }
        obj.set(cx, "txs", txs_array)?;

        Ok(())
    }
}

impl NeonJsSerialize for StacksBlockHeader {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let version = cx.number(self.version);
        obj.set(cx, "version", version)?;

        // Total work
        let total_work_obj = cx.empty_object();
        self.total_work.neon_js_serialize(cx, &total_work_obj, &())?;
        obj.set(cx, "total_work", total_work_obj)?;

        // VRF proof
        let proof = cx.string(encode_hex(&self.proof.0));
        obj.set(cx, "proof", proof)?;

        let parent_block = cx.string(encode_hex(&self.parent_block.0));
        obj.set(cx, "parent_block", parent_block)?;

        let parent_microblock = cx.string(encode_hex(&self.parent_microblock.0));
        obj.set(cx, "parent_microblock", parent_microblock)?;

        let parent_microblock_sequence = cx.number(self.parent_microblock_sequence);
        obj.set(cx, "parent_microblock_sequence", parent_microblock_sequence)?;

        let tx_merkle_root = cx.string(encode_hex(&self.tx_merkle_root.0));
        obj.set(cx, "tx_merkle_root", tx_merkle_root)?;

        let state_index_root = cx.string(encode_hex(&self.state_index_root.0));
        obj.set(cx, "state_index_root", state_index_root)?;

        let microblock_pubkey_hash = cx.string(encode_hex(&self.microblock_pubkey_hash));
        obj.set(cx, "microblock_pubkey_hash", microblock_pubkey_hash)?;

        // Computed block hash
        let block_hash = cx.string(encode_hex(&self.block_hash()));
        obj.set(cx, "block_hash", block_hash)?;

        Ok(())
    }
}

impl NeonJsSerialize for StacksWorkScore {
    fn neon_js_serialize(
        &self,
        cx: &mut FunctionContext,
        obj: &Handle<JsObject>,
        _extra_ctx: &(),
    ) -> NeonResult<()> {
        let burn = cx.string(self.burn.to_string());
        obj.set(cx, "burn", burn)?;

        let work = cx.string(self.work.to_string());
        obj.set(cx, "work", work)?;

        Ok(())
    }
}
