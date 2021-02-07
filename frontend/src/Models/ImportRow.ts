import { Transaction } from "./Transaction";
import { Rule } from "./Rule";

export interface RealTransaction {
  [key: string]: any;
}

export type RealTransactionField = keyof RealTransaction;
export type TransactionField = keyof Transaction;

export interface ImportRow {
  real_transaction?: RealTransaction;
  recorded_transaction?: Transaction;
  rule?: Rule;
}
