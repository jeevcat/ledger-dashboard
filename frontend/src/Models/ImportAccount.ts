import { RealTransactionField } from "./ImportRow";

export interface ImportAccount {
  humanName: string;
  path: string;
  defaultColumns: RealTransactionField[];
}

export const ImportAccounts: ImportAccount[] = [
  {
    humanName: "N26",
    path: "n26",
    defaultColumns: ["visibleTS", "referenceText", "partnerName", "merchantName", "amount"],
  },
  {
    humanName: "Interactive Brokers",
    path: "ib",
    defaultColumns: ["Date", "Description", "Currency", "Amount"],
  },
];
