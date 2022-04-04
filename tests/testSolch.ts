import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TestSolch } from "../target/types/test_solch";

describe("testSolch", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.TestSolch as Program<TestSolch>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
