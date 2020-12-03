import { NativeClarityBinProvider } from "@blockstack/clarity";
import { expect } from "chai";
import { getTempFilePath } from "@blockstack/clarity/lib/utils/fsUtil";
import { getDefaultBinaryFilePath } from "@blockstack/clarity-native-bin";
import { BNSClient, PriceFunction } from "../src/bns-client";
import { mineBlocks } from "./utils";

describe("BNS Test Suite - NAME_UPDATE", () => {
  let bns: BNSClient;
  let provider: NativeClarityBinProvider;

  const addresses = [
    "SP2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKNRV9EJ7",
    "S02J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKPVKG2CE",
    "SZ2J6ZY48GV1EZ5V2V5RB9MP66SW86PYKKQ9H6DPR",
    "SPMQEKN07D1VHAB8XQV835E3PTY3QWZRZ5H0DM36"
  ];
  const alice = addresses[0];
  const bob = addresses[1];
  const charlie = addresses[2];
  const dave = addresses[3];

  const cases = [{
    namespace: "blockstack",
    version: 1,
    salt: "0000",
    value: 96,
    namespaceOwner: alice,
    nameOwner: bob,
    priceFunction: {
      buckets: [7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
      base: 4,
      coeff: 250,
      noVoyelDiscount: 4,
      nonAlphaDiscount: 4,
    },
    renewalRule: 4294967295,
    nameImporter: alice,
    zonefile: "0000",
  }, {
    namespace: "id",
    version: 1,
    salt: "0000",
    value: 9600,
    namespaceOwner: alice,
    nameOwner: bob,
    priceFunction: {
      buckets: [6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
      base: 4,
      coeff: 250,
      noVoyelDiscount: 20,
      nonAlphaDiscount: 20,
    },
    renewalRule: 52595,
    nameImporter: alice,
    zonefile: "1111",
  }];

  beforeEach(async () => {
    const allocations = [
      { principal: alice, amount: 10_000_000_000 },
      { principal: bob, amount: 10_000_000 },
      { principal: charlie, amount: 10_000_000 },
      { principal: dave, amount: 10_000_000 },
    ]
    const binFile = getDefaultBinaryFilePath();
    const dbFileName = getTempFilePath();
    provider = await NativeClarityBinProvider.create(allocations, dbFileName, binFile);
    bns = new BNSClient(provider);
    await bns.deployContract();
  });


  it("Given an unlaunched namespace 'blockstack', owned by Alice", async () => {
    let block_height = 2;
    let namespace_preorder_ttl = 10;
    let name_preorder_ttl = 10;


      var receipt = await bns.namespacePreorder(cases[0].namespace, cases[0].salt, cases[0].value, { sender: cases[0].namespaceOwner });
      expect(receipt.success).eq(true);
      expect(receipt.result).include(`${block_height+namespace_preorder_ttl}`);
      block_height += 1;

      receipt = await bns.namespaceReveal(
        cases[0].namespace, 
        cases[0].version, 
        cases[0].salt,
        cases[0].priceFunction, 
        cases[0].renewalRule, 
        cases[0].nameImporter, { sender: cases[0].namespaceOwner });
      expect(receipt.success).eq(true);
      expect(receipt.result).include('true');
    // });

    // it("should be possible for Alice to import a name", async () => {
      receipt = await bns.nameImport(cases[0].namespace, "alice", "4444", { sender: alice });
      expect(receipt.success).eq(true);
      expect(receipt.result).include('true');
    // })

    // it("should be possible for Alice to update her domain", async () => {
      receipt = await bns.nameUpdate(
        cases[0].namespace, 
        "alice", 
        "5555", { sender: alice });
      expect(receipt.error).include('1007');
      expect(receipt.success).eq(false);
    // });

    // it("should not be possible for Bob to import a name", async () => {
      receipt = await bns.nameImport(cases[0].namespace, "bob", "4444", { sender: bob });
      expect(receipt.success).eq(false);
      expect(receipt.error).include('1011');
    // })

    // it("should not resolve (namespace not launched yet)", async () => {
      receipt = await bns.getNameZonefile(
        cases[0].namespace, 
        "alice", { sender: cases[0].nameOwner });
      expect(receipt.result).include('0x34343434');
      expect(receipt.success).eq(true);
    // });

    // describe("When Alice is launching the namespace 'blockstack' at block #20", () => {

    //   beforeEach(async () => {
        receipt = await bns.namespaceReady(cases[0].namespace, { sender: alice });
        expect(receipt.success).eq(true);
        expect(receipt.result).include('true');  
      // });

      // it("Resolving 'alice.blockstack' should succeed", async () => {
        receipt = await bns.getNameZonefile(
          cases[0].namespace, 
          "alice", { sender: cases[0].nameOwner });
        expect(receipt.result).include('0x34343434');
        expect(receipt.success).eq(true);
      // });  


      // it("Bob preordering 'bob.blockstack' waiting for the namespace to be launched should fail", async () => {

        receipt = await bns.namePreorder(
          cases[0].namespace,
          "bob",
          cases[0].salt, 
          2560000, { sender: cases[0].nameOwner });
        expect(receipt.success).eq(true);
        expect(receipt.result).include('u20');
  
        receipt = await bns.nameRegister(
          cases[0].namespace, 
          "bob", 
          cases[0].salt, 
          cases[0].zonefile, { sender: cases[0].nameOwner });
        expect(receipt.result).include('true');
        expect(receipt.success).eq(true);
      // });

      // describe("Bob updating his zonefile - from 1111 to 2222", () => {

      //   it("should succeed", async () => {
          
          receipt = await bns.nameUpdate(
            cases[0].namespace, 
            "bob", 
            "2222", { sender: cases[0].nameOwner });
          expect(receipt.result).include('true');
          expect(receipt.success).eq(true);
        // });

        // it("should resolve as expected", async () => {
          receipt = await bns.getNameZonefile(
            cases[0].namespace, 
            "bob", { sender: cases[0].nameOwner });
          expect(receipt.result).include('0x32323232');
          expect(receipt.success).eq(true);
      //   });
      // });

      // describe("Charlie updating Bob's zonefile - from 2222 to 3333", () => {

      //   it("should fail", async () => {
          
          receipt = await bns.nameUpdate(
            cases[0].namespace, 
            "bob", 
            "3333", { sender: charlie });
          expect(receipt.error).include('2006');
          expect(receipt.success).eq(false);
        // });

        // it("should resolve as expected", async () => {
          receipt = await bns.getNameZonefile(
            cases[0].namespace, 
            "bob", { sender: cases[0].nameOwner });
          expect(receipt.result).include('0x32323232');
          expect(receipt.success).eq(true);
    //     });
    //   });

    // });
  });
});

