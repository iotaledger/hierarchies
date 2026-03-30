// Copyright (c) 2026 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Shared utilities for Access Controller Bridge TypeScript examples.

import { Transaction } from "@iota/iota-sdk/transactions";
import { IotaClient } from "@iota/iota-sdk/client";
import { Ed25519Keypair } from "@iota/iota-sdk/keypairs/ed25519";
import { getFaucetHost, requestIotaFromFaucetV0 } from "@iota/iota-sdk/faucet";

// ===== Environment =====

const NETWORK_URL = process.env.NETWORK_URL || "http://127.0.0.1:9000";
const NETWORK_NAME_FAUCET = process.env.NETWORK_NAME_FAUCET || "localnet";

function envPkg(name: string): string {
    const val = process.env[name];
    if (!val) throw new Error(`${name} env variable must be set`);
    return val;
}

export const HIER_PKG = () => envPkg("IOTA_HIERARCHIES_PKG_ID");
export const AT_PKG = () => envPkg("IOTA_AUDIT_TRAIL_PKG_ID");
export const ACB_PKG = () => envPkg("IOTA_ACB_PKG_ID");
export const TF_PKG = () => envPkg("IOTA_TF_COMPONENTS_PKG_ID");

const CLOCK_ID = "0x6";

// ===== Client & signer =====

export async function getClientAndSigner(): Promise<{ client: IotaClient; keypair: Ed25519Keypair; address: string }> {
    const client = new IotaClient({ url: NETWORK_URL });
    const keypair = Ed25519Keypair.generate();
    const address = keypair.toIotaAddress();

    await requestIotaFromFaucetV0({
        host: getFaucetHost(NETWORK_NAME_FAUCET),
        recipient: address,
    });

    return { client, keypair, address };
}

// ===== Transaction execution =====

export interface ObjectChange {
    type: string;
    objectType?: string;
    objectId?: string;
}

export async function executeTx(
    client: IotaClient,
    keypair: Ed25519Keypair,
    tx: Transaction,
): Promise<ObjectChange[]> {
    const result = await client.signAndExecuteTransaction({
        transaction: tx,
        signer: keypair,
        options: { showEffects: true, showObjectChanges: true },
    });

    if (result.effects?.status?.status !== "success") {
        throw new Error(`Transaction failed: ${JSON.stringify(result.effects?.status)}`);
    }

    // Wait for indexer to catch up before subsequent transactions
    await client.waitForTransaction({ digest: result.digest });

    return (result.objectChanges || []) as ObjectChange[];
}

export function findCreated(changes: ObjectChange[], typeContains: string): string[] {
    return changes
        .filter((c) => c.type === "created" && c.objectType?.includes(typeContains))
        .map((c) => c.objectId!)
        .filter(Boolean);
}

// ===== Shared object helper =====

async function sharedArg(client: IotaClient, id: string, mutable: boolean) {
    const obj = await client.getObject({ id, options: { showOwner: true } });
    const owner = obj.data?.owner as any;
    if (!owner?.Shared) throw new Error(`Object ${id} is not shared`);
    return {
        objectId: id,
        initialSharedVersion: owner.Shared.initial_shared_version,
        mutable,
    };
}

// ===== PTB Helper =====

export class PtbHelper {
    hierPkg: string;
    atPkg: string;
    acbPkg: string;
    tfPkg: string;

    constructor() {
        this.hierPkg = HIER_PKG();
        this.atPkg = AT_PKG();
        this.acbPkg = ACB_PKG();
        this.tfPkg = TF_PKG();
    }

    // -- Property construction --

    propName(tx: Transaction, name: string): any {
        return tx.moveCall({
            target: `${this.hierPkg}::property_name::new_property_name`,
            arguments: [tx.pure.string(name)],
        });
    }

    propValueStr(tx: Transaction, val: string): any {
        return tx.moveCall({
            target: `${this.hierPkg}::property_value::new_property_value_string`,
            arguments: [tx.pure.string(val)],
        });
    }

    // -- VecMap<PropertyName, PropertyValue> --

    propertyMap(tx: Transaction, pairs: [string, string][]): any {
        const keys = pairs.map(([k]) => this.propName(tx, k));
        const vals = pairs.map(([, v]) => this.propValueStr(tx, v));

        const kt = `${this.hierPkg}::property_name::PropertyName`;
        const vt = `${this.hierPkg}::property_value::PropertyValue`;
        const kVec = tx.makeMoveVec({ type: kt, elements: keys });
        const vVec = tx.makeMoveVec({ type: vt, elements: vals });

        return tx.moveCall({
            target: `${this.hierPkg}::utils::vec_map_from_keys_values`,
            typeArguments: [kt, vt],
            arguments: [kVec, vVec],
        });
    }

    // -- FederationProperty --

    fedProperty(tx: Transaction, name: string, values: string[], allowAny: boolean): any {
        const pn = this.propName(tx, name);
        const vt = `${this.hierPkg}::property_value::PropertyValue`;
        const valArgs = values.map((v) => this.propValueStr(tx, v));
        const vVec = tx.makeMoveVec({ type: vt, elements: valArgs });
        const aset = tx.moveCall({
            target: `${this.hierPkg}::utils::create_vec_set`,
            typeArguments: [vt],
            arguments: [vVec],
        });
        const shapeT = `${this.hierPkg}::property_shape::PropertyShape`;
        const sn = tx.moveCall({
            target: "0x1::option::none",
            typeArguments: [shapeT],
            arguments: [],
        });
        return tx.moveCall({
            target: `${this.hierPkg}::property::new_property`,
            arguments: [pn, aset, tx.pure.bool(allowAny), sn],
        });
    }

    // -- Federation --

    newFederation(tx: Transaction): void {
        tx.moveCall({
            target: `${this.hierPkg}::main::new_federation`,
            arguments: [],
        });
    }

    fedAddProperty(
        tx: Transaction,
        fedArg: any | string,
        capArg: any | string,
        name: string,
        values: string[],
        allowAny: boolean,
    ): void {
        const prop = this.fedProperty(tx, name, values, allowAny);
        tx.moveCall({
            target: `${this.hierPkg}::main::add_property`,
            arguments: [typeof fedArg === "string" ? tx.object(fedArg) : fedArg, typeof capArg === "string" ? tx.object(capArg) : capArg, prop],
        });
    }

    fedAccreditToAttest(
        tx: Transaction,
        fedArg: any | string,
        accreditCap: any | string,
        receiver: string,
        props: { name: string; values: string[]; allowAny: boolean }[],
    ): void {
        const propT = `${this.hierPkg}::property::FederationProperty`;
        const propArgs = props.map((p) => this.fedProperty(tx, p.name, p.values, p.allowAny));
        const pv = tx.makeMoveVec({ type: propT, elements: propArgs });
        tx.moveCall({
            target: `${this.hierPkg}::main::create_accreditation_to_attest`,
            arguments: [
                typeof fedArg === "string" ? tx.object(fedArg) : fedArg,
                typeof accreditCap === "string" ? tx.object(accreditCap) : accreditCap,
                tx.pure.address(receiver),
                pv,
                tx.object(CLOCK_ID),
            ],
        });
    }

    // -- Audit Trail --

    trailCreate(tx: Transaction, name: string, sender: string): void {
        const dataT = `${this.atPkg}::record::Data`;
        const irT = `${this.atPkg}::record::InitialRecord<${dataT}>`;
        const initial = tx.moveCall({ target: "0x1::option::none", typeArguments: [irT], arguments: [] });
        const wn = tx.moveCall({ target: `${this.atPkg}::locking::window_none`, arguments: [] });
        const tl1 = tx.moveCall({ target: `${this.tfPkg}::timelock::none`, arguments: [] });
        const tl2 = tx.moveCall({ target: `${this.tfPkg}::timelock::none`, arguments: [] });
        const locking = tx.moveCall({ target: `${this.atPkg}::locking::new`, arguments: [wn, tl1, tl2] });
        const mdT = `${this.atPkg}::main::ImmutableMetadata`;
        const nm = tx.pure.string(name);
        const desc = tx.pure.option("string", null);
        const md = tx.moveCall({ target: `${this.atPkg}::main::new_trail_metadata`, arguments: [nm, desc] });
        const trailMd = tx.moveCall({ target: "0x1::option::some", typeArguments: [mdT], arguments: [md] });
        const updNone = tx.pure.option("string", null);
        const tags = tx.pure.vector("string", []);
        const result = tx.moveCall({
            target: `${this.atPkg}::main::create`,
            typeArguments: [dataT],
            arguments: [initial, locking, trailMd, updNone, tags, tx.object(CLOCK_ID)],
        });
        // create() returns (Capability, ID). Transfer admin_cap to sender, discard ID.
        tx.transferObjects([result[0]], sender);
    }

    trailCreateRole(
        tx: Transaction,
        trailArg: any | string,
        adminCap: any | string,
        roleName: string,
        permissionFns: string[],
    ): void {
        const permT = `${this.atPkg}::permission::Permission`;
        const perms = permissionFns.map((f) =>
            tx.moveCall({ target: `${this.atPkg}::permission::${f}`, arguments: [] })
        );
        const pv = tx.makeMoveVec({ type: permT, elements: perms });
        const ps = tx.moveCall({ target: `${this.atPkg}::permission::from_vec`, arguments: [pv] });
        const rtT = `${this.atPkg}::record_tags::RoleTags`;
        const rtNone = tx.moveCall({ target: "0x1::option::none", typeArguments: [rtT], arguments: [] });
        tx.moveCall({
            target: `${this.atPkg}::main::create_role`,
            typeArguments: [`${this.atPkg}::record::Data`],
            arguments: [
                typeof trailArg === "string" ? tx.object(trailArg) : trailArg,
                typeof adminCap === "string" ? tx.object(adminCap) : adminCap,
                tx.pure.string(roleName),
                ps,
                rtNone,
                tx.object(CLOCK_ID),
            ],
        });
    }

    trailMintCapability(
        tx: Transaction,
        trailArg: any | string,
        adminCap: any | string,
        roleName: string,
    ): void {
        tx.moveCall({
            target: `${this.atPkg}::main::new_capability`,
            typeArguments: [`${this.atPkg}::record::Data`],
            arguments: [
                typeof trailArg === "string" ? tx.object(trailArg) : trailArg,
                typeof adminCap === "string" ? tx.object(adminCap) : adminCap,
                tx.pure.string(roleName),
                tx.pure.option("address", null),
                tx.pure.option("u64", null),
                tx.pure.option("u64", null),
                tx.object(CLOCK_ID),
            ],
        });
    }

    trailAddRecord(
        tx: Transaction,
        trailArg: any | string,
        cap: any,
        text: string,
    ): void {
        const data = tx.moveCall({
            target: `${this.atPkg}::record::new_text`,
            arguments: [tx.pure.string(text)],
        });
        tx.moveCall({
            target: `${this.atPkg}::main::add_record`,
            typeArguments: [`${this.atPkg}::record::Data`],
            arguments: [
                typeof trailArg === "string" ? tx.object(trailArg) : trailArg,
                cap,
                data,
                tx.pure.option("string", null),
                tx.pure.option("string", null),
                tx.object(CLOCK_ID),
            ],
        });
    }

    // -- ACB --

    acbCreate(
        tx: Transaction,
        fedArg: any | string,
        targetId: string,
        roles: { name: string; properties: [string, string][] }[],
    ): any {
        const strT = "0x1::string::String";
        const cfgT = `${this.acbPkg}::bridge::RoleConfig`;

        const keys = roles.map((r) => tx.pure.string(r.name));
        const vals = roles.map((r) => {
            const propMap = this.propertyMap(tx, r.properties);
            return tx.moveCall({
                target: `${this.acbPkg}::bridge::new_role_config`,
                arguments: [propMap],
            });
        });

        const kVec = tx.makeMoveVec({ type: strT, elements: keys });
        const vVec = tx.makeMoveVec({ type: cfgT, elements: vals });
        const configs = tx.moveCall({
            target: `${this.hierPkg}::utils::vec_map_from_keys_values`,
            typeArguments: [strT, cfgT],
            arguments: [kVec, vVec],
        });

        return tx.moveCall({
            target: `${this.acbPkg}::bridge::create`,
            typeArguments: ["bool"],
            arguments: [
                typeof fedArg === "string" ? tx.object(fedArg) : fedArg,
                tx.pure.address(targetId),
                configs,
            ],
        });
    }

    acbShare(tx: Transaction, acb: any): void {
        tx.moveCall({
            target: "0x2::transfer::public_share_object",
            typeArguments: [`${this.acbPkg}::bridge::AccessControllerBridge<bool>`],
            arguments: [acb],
        });
    }

    acbDeposit(
        tx: Transaction,
        acbArg: any | string,
        fedArg: any | string,
        roleName: string,
        capArg: any | string,
    ): void {
        tx.moveCall({
            target: `${this.acbPkg}::bridge::deposit_capability`,
            typeArguments: ["bool"],
            arguments: [
                typeof acbArg === "string" ? tx.object(acbArg) : acbArg,
                typeof fedArg === "string" ? tx.object(fedArg) : fedArg,
                tx.pure.string(roleName),
                typeof capArg === "string" ? tx.object(capArg) : capArg,
            ],
        });
    }

    acbBorrow(
        tx: Transaction,
        acbArg: any | string,
        fedArg: any | string,
        roleName: string,
    ): { cap: any; receipt: any } {
        const context = tx.moveCall({
            target: `${this.acbPkg}::bridge::role_name`,
            arguments: [tx.pure.string(roleName)],
        });
        const result = tx.moveCall({
            target: `${this.acbPkg}::bridge::borrow`,
            typeArguments: ["bool"],
            arguments: [
                typeof acbArg === "string" ? tx.object(acbArg) : acbArg,
                typeof fedArg === "string" ? tx.object(fedArg) : fedArg,
                context,
                tx.object(CLOCK_ID),
            ],
        });
        return { cap: result[0], receipt: result[1] };
    }

    acbReturn(
        tx: Transaction,
        acbArg: any | string,
        cap: any,
        receipt: any,
    ): void {
        tx.moveCall({
            target: `${this.acbPkg}::bridge::return_cap`,
            typeArguments: ["bool"],
            arguments: [
                typeof acbArg === "string" ? tx.object(acbArg) : acbArg,
                cap,
                receipt,
                tx.object(CLOCK_ID),
            ],
        });
    }
}

// ===== Shared arg wrappers for executeTx =====

export async function sharedMut(client: IotaClient, tx: Transaction, id: string) {
    const info = await sharedArg(client, id, true);
    return tx.sharedObjectRef(info);
}

export async function sharedImm(client: IotaClient, tx: Transaction, id: string) {
    const info = await sharedArg(client, id, false);
    return tx.sharedObjectRef(info);
}
