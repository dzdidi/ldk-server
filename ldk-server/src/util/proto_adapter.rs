use bytes::Bytes;
use hex::prelude::*;
use ldk_node::config::{ChannelConfig, MaxDustHTLCExposure};
use ldk_node::payment::{PaymentDetails, PaymentDirection, PaymentKind, PaymentStatus};
use ldk_node::{ChannelDetails, LightningBalance, PendingSweepBalance};
use ldk_server_protos::types::lightning_balance::BalanceType::{
	ClaimableAwaitingConfirmations, ClaimableOnChannelClose, ContentiousClaimable,
	CounterpartyRevokedOutputClaimable, MaybePreimageClaimableHtlc, MaybeTimeoutClaimableHtlc,
};
use ldk_server_protos::types::payment_kind::Kind::{
	Bolt11, Bolt11Jit, Bolt12Offer, Bolt12Refund, Onchain, Spontaneous,
};
use ldk_server_protos::types::pending_sweep_balance::BalanceType::{
	AwaitingThresholdConfirmations, BroadcastAwaitingConfirmation, PendingBroadcast,
};
use ldk_server_protos::types::{Channel, LspFeeLimits, OutPoint, Payment};

pub(crate) fn channel_to_proto(channel: ChannelDetails) -> Channel {
	Channel {
		channel_id: channel.channel_id.0.to_lower_hex_string(),
		counterparty_node_id: channel.counterparty_node_id.to_string(),
		funding_txo: channel
			.funding_txo
			.map(|o| OutPoint { txid: o.txid.to_string(), vout: o.vout }),
		user_channel_id: channel.user_channel_id.0.to_string(),
		unspendable_punishment_reserve: channel.unspendable_punishment_reserve,
		channel_value_sats: channel.channel_value_sats,
		feerate_sat_per_1000_weight: channel.feerate_sat_per_1000_weight,
		outbound_capacity_msat: channel.outbound_capacity_msat,
		inbound_capacity_msat: channel.inbound_capacity_msat,
		confirmations_required: channel.confirmations_required,
		confirmations: channel.confirmations,
		is_outbound: channel.is_outbound,
		is_channel_ready: channel.is_channel_ready,
		is_usable: channel.is_usable,
		is_announced: channel.is_announced,
		channel_config: Some(channel_config_to_proto(channel.config)),
		next_outbound_htlc_limit_msat: channel.next_outbound_htlc_limit_msat,
		next_outbound_htlc_minimum_msat: channel.next_outbound_htlc_minimum_msat,
		force_close_spend_delay: channel.force_close_spend_delay.map(|x| x as u32),
		counterparty_outbound_htlc_minimum_msat: channel.counterparty_outbound_htlc_minimum_msat,
		counterparty_outbound_htlc_maximum_msat: channel.counterparty_outbound_htlc_maximum_msat,
		counterparty_unspendable_punishment_reserve: channel
			.counterparty_unspendable_punishment_reserve,
		counterparty_forwarding_info_fee_base_msat: channel
			.counterparty_forwarding_info_fee_base_msat,
		counterparty_forwarding_info_fee_proportional_millionths: channel
			.counterparty_forwarding_info_fee_proportional_millionths,
		counterparty_forwarding_info_cltv_expiry_delta: channel
			.counterparty_forwarding_info_cltv_expiry_delta
			.map(|x| x as u32),
	}
}

pub(crate) fn channel_config_to_proto(
	channel_config: ChannelConfig,
) -> ldk_server_protos::types::ChannelConfig {
	ldk_server_protos::types::ChannelConfig {
		forwarding_fee_proportional_millionths: Some(
			channel_config.forwarding_fee_proportional_millionths,
		),
		forwarding_fee_base_msat: Some(channel_config.forwarding_fee_base_msat),
		cltv_expiry_delta: Some(channel_config.cltv_expiry_delta as u32),
		force_close_avoidance_max_fee_satoshis: Some(
			channel_config.force_close_avoidance_max_fee_satoshis,
		),
		accept_underpaying_htlcs: Some(channel_config.accept_underpaying_htlcs),
		max_dust_htlc_exposure: match channel_config.max_dust_htlc_exposure {
			MaxDustHTLCExposure::FixedLimit { limit_msat } => {
				Some(ldk_server_protos::types::channel_config::MaxDustHtlcExposure::FixedLimitMsat(
					limit_msat,
				))
			},
			MaxDustHTLCExposure::FeeRateMultiplier { multiplier } => Some(
				ldk_server_protos::types::channel_config::MaxDustHtlcExposure::FeeRateMultiplier(
					multiplier,
				),
			),
		},
	}
}

pub(crate) fn payment_to_proto(payment: PaymentDetails) -> Payment {
	Payment {
		id: payment.id.0.to_lower_hex_string(),
		kind: Some(payment_kind_to_proto(payment.kind)),
		amount_msat: payment.amount_msat,
		direction: match payment.direction {
			PaymentDirection::Inbound => ldk_server_protos::types::PaymentDirection::Inbound.into(),
			PaymentDirection::Outbound => {
				ldk_server_protos::types::PaymentDirection::Outbound.into()
			},
		},
		status: match payment.status {
			PaymentStatus::Pending => ldk_server_protos::types::PaymentStatus::Pending.into(),
			PaymentStatus::Succeeded => ldk_server_protos::types::PaymentStatus::Succeeded.into(),
			PaymentStatus::Failed => ldk_server_protos::types::PaymentStatus::Failed.into(),
		},
		latest_update_timestamp: payment.latest_update_timestamp,
	}
}

pub(crate) fn payment_kind_to_proto(
	payment_kind: PaymentKind,
) -> ldk_server_protos::types::PaymentKind {
	match payment_kind {
		PaymentKind::Onchain => ldk_server_protos::types::PaymentKind {
			kind: Some(Onchain(ldk_server_protos::types::Onchain {})),
		},
		PaymentKind::Bolt11 { hash, preimage, secret } => ldk_server_protos::types::PaymentKind {
			kind: Some(Bolt11(ldk_server_protos::types::Bolt11 {
				hash: hash.to_string(),
				preimage: preimage.map(|p| p.to_string()),
				secret: secret.map(|s| Bytes::copy_from_slice(&s.0)),
			})),
		},
		PaymentKind::Bolt11Jit { hash, preimage, secret, lsp_fee_limits } => {
			ldk_server_protos::types::PaymentKind {
				kind: Some(Bolt11Jit(ldk_server_protos::types::Bolt11Jit {
					hash: hash.to_string(),
					preimage: preimage.map(|p| p.to_string()),
					secret: secret.map(|s| Bytes::copy_from_slice(&s.0)),
					lsp_fee_limits: Some(LspFeeLimits {
						max_total_opening_fee_msat: lsp_fee_limits.max_total_opening_fee_msat,
						max_proportional_opening_fee_ppm_msat: lsp_fee_limits
							.max_proportional_opening_fee_ppm_msat,
					}),
				})),
			}
		},
		PaymentKind::Bolt12Offer { hash, preimage, secret, offer_id, payer_note, quantity } => {
			ldk_server_protos::types::PaymentKind {
				kind: Some(Bolt12Offer(ldk_server_protos::types::Bolt12Offer {
					hash: hash.map(|h| h.to_string()),
					preimage: preimage.map(|p| p.to_string()),
					secret: secret.map(|s| Bytes::copy_from_slice(&s.0)),
					offer_id: offer_id.0.to_lower_hex_string(),
					payer_note: payer_note.map(|s| s.to_string()),
					quantity,
				})),
			}
		},
		PaymentKind::Bolt12Refund { hash, preimage, secret, payer_note, quantity } => {
			ldk_server_protos::types::PaymentKind {
				kind: Some(Bolt12Refund(ldk_server_protos::types::Bolt12Refund {
					hash: hash.map(|h| h.to_string()),
					preimage: preimage.map(|p| p.to_string()),
					secret: secret.map(|s| Bytes::copy_from_slice(&s.0)),
					payer_note: payer_note.map(|s| s.to_string()),
					quantity,
				})),
			}
		},
		PaymentKind::Spontaneous { hash, preimage } => ldk_server_protos::types::PaymentKind {
			kind: Some(Spontaneous(ldk_server_protos::types::Spontaneous {
				hash: hash.to_string(),
				preimage: preimage.map(|p| p.to_string()),
			})),
		},
	}
}

pub(crate) fn lightning_balance_to_proto(
	lightning_balance: LightningBalance,
) -> ldk_server_protos::types::LightningBalance {
	match lightning_balance {
		LightningBalance::ClaimableOnChannelClose {
			channel_id,
			counterparty_node_id,
			amount_satoshis,
			transaction_fee_satoshis,
			outbound_payment_htlc_rounded_msat,
			outbound_forwarded_htlc_rounded_msat,
			inbound_claiming_htlc_rounded_msat,
			inbound_htlc_rounded_msat,
		} => ldk_server_protos::types::LightningBalance {
			balance_type: Some(ClaimableOnChannelClose(
				ldk_server_protos::types::ClaimableOnChannelClose {
					channel_id: channel_id.0.to_lower_hex_string(),
					counterparty_node_id: counterparty_node_id.to_string(),
					amount_satoshis,
					transaction_fee_satoshis,
					outbound_payment_htlc_rounded_msat,
					outbound_forwarded_htlc_rounded_msat,
					inbound_claiming_htlc_rounded_msat,
					inbound_htlc_rounded_msat,
				},
			)),
		},
		LightningBalance::ClaimableAwaitingConfirmations {
			channel_id,
			counterparty_node_id,
			amount_satoshis,
			confirmation_height,
			..
		} => ldk_server_protos::types::LightningBalance {
			balance_type: Some(ClaimableAwaitingConfirmations(
				ldk_server_protos::types::ClaimableAwaitingConfirmations {
					channel_id: channel_id.0.to_lower_hex_string(),
					counterparty_node_id: counterparty_node_id.to_string(),
					amount_satoshis,
					confirmation_height,
				},
			)),
		},
		LightningBalance::ContentiousClaimable {
			channel_id,
			counterparty_node_id,
			amount_satoshis,
			timeout_height,
			payment_hash,
			payment_preimage,
		} => ldk_server_protos::types::LightningBalance {
			balance_type: Some(ContentiousClaimable(
				ldk_server_protos::types::ContentiousClaimable {
					channel_id: channel_id.0.to_lower_hex_string(),
					counterparty_node_id: counterparty_node_id.to_string(),
					amount_satoshis,
					timeout_height,
					payment_hash: payment_hash.to_string(),
					payment_preimage: payment_preimage.to_string(),
				},
			)),
		},
		LightningBalance::MaybeTimeoutClaimableHTLC {
			channel_id,
			counterparty_node_id,
			amount_satoshis,
			claimable_height,
			payment_hash,
			outbound_payment,
		} => ldk_server_protos::types::LightningBalance {
			balance_type: Some(MaybeTimeoutClaimableHtlc(
				ldk_server_protos::types::MaybeTimeoutClaimableHtlc {
					channel_id: channel_id.0.to_lower_hex_string(),
					counterparty_node_id: counterparty_node_id.to_string(),
					amount_satoshis,
					claimable_height,
					payment_hash: payment_hash.to_string(),
					outbound_payment,
				},
			)),
		},
		LightningBalance::MaybePreimageClaimableHTLC {
			channel_id,
			counterparty_node_id,
			amount_satoshis,
			expiry_height,
			payment_hash,
		} => ldk_server_protos::types::LightningBalance {
			balance_type: Some(MaybePreimageClaimableHtlc(
				ldk_server_protos::types::MaybePreimageClaimableHtlc {
					channel_id: channel_id.0.to_lower_hex_string(),
					counterparty_node_id: counterparty_node_id.to_string(),
					amount_satoshis,
					expiry_height,
					payment_hash: payment_hash.to_string(),
				},
			)),
		},
		LightningBalance::CounterpartyRevokedOutputClaimable {
			channel_id,
			counterparty_node_id,
			amount_satoshis,
		} => ldk_server_protos::types::LightningBalance {
			balance_type: Some(CounterpartyRevokedOutputClaimable(
				ldk_server_protos::types::CounterpartyRevokedOutputClaimable {
					channel_id: channel_id.0.to_lower_hex_string(),
					counterparty_node_id: counterparty_node_id.to_string(),
					amount_satoshis,
				},
			)),
		},
	}
}

pub(crate) fn pending_sweep_balance_to_proto(
	pending_sweep_balance: PendingSweepBalance,
) -> ldk_server_protos::types::PendingSweepBalance {
	match pending_sweep_balance {
		PendingSweepBalance::PendingBroadcast { channel_id, amount_satoshis } => {
			ldk_server_protos::types::PendingSweepBalance {
				balance_type: Some(PendingBroadcast(ldk_server_protos::types::PendingBroadcast {
					channel_id: channel_id.map(|c| c.0.to_lower_hex_string()),
					amount_satoshis,
				})),
			}
		},
		PendingSweepBalance::BroadcastAwaitingConfirmation {
			channel_id,
			latest_broadcast_height,
			latest_spending_txid,
			amount_satoshis,
		} => ldk_server_protos::types::PendingSweepBalance {
			balance_type: Some(BroadcastAwaitingConfirmation(
				ldk_server_protos::types::BroadcastAwaitingConfirmation {
					channel_id: channel_id.map(|c| c.0.to_lower_hex_string()),
					latest_broadcast_height,
					latest_spending_txid: latest_spending_txid.to_string(),
					amount_satoshis,
				},
			)),
		},
		PendingSweepBalance::AwaitingThresholdConfirmations {
			channel_id,
			latest_spending_txid,
			confirmation_hash,
			confirmation_height,
			amount_satoshis,
		} => ldk_server_protos::types::PendingSweepBalance {
			balance_type: Some(AwaitingThresholdConfirmations(
				ldk_server_protos::types::AwaitingThresholdConfirmations {
					channel_id: channel_id.map(|c| c.0.to_lower_hex_string()),
					latest_spending_txid: latest_spending_txid.to_string(),
					confirmation_hash: confirmation_hash.to_string(),
					confirmation_height,
					amount_satoshis,
				},
			)),
		},
	}
}