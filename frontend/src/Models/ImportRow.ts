import { HledgerTransaction } from "./HledgerTransaction";
import { Rule } from "./Rule";

export interface RealTransaction {
  [key: string]: any;
}

export type RealTransactionField = keyof RealTransaction;
export type TransactionField = keyof HledgerTransaction;

export interface TransactionResponse {
  real_transaction: RealTransaction;
  hledger_transaction?: HledgerTransaction;
  rule?: Rule;
}

export interface ExistingTransactionResponse {
  real_transaction: RealTransaction | null;
  hledger_transaction: HledgerTransaction;
  real_cumulative: number;
  hledger_cumulative: number;
  errors: string[];
}
