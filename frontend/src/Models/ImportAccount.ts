import ib from "../Images/ib.png";
import ing from "../Images/ing.png";
import n26 from "../Images/n26.png";
import { RealTransaction, RealTransactionField } from "./ImportRow";

export type ImportAccountType = "ing" | "n26" | "ib";

export interface ImportAccount {
  humanName: string;
  id: ImportAccountType;
  icon: string;
  dateColumn: RealTransactionField;
  amountColumns: RealTransactionField[];
  defaultColumns: RealTransactionField[];
}

export const ImportAccounts: ImportAccount[] = [
  {
    humanName: "ING DiBa",
    id: "ing",
    icon: ing,
    dateColumn: "made_on",
    amountColumns: ["amount"],
    defaultColumns: ["currency", "description", "payee", "category"],
  },
  {
    humanName: "N26",
    id: "n26",
    icon: n26,
    dateColumn: "visibleTS",
    amountColumns: ["amount"],
    defaultColumns: ["currency", "referenceText", "partnerName", "merchantName", "mcc"],
  },
  {
    humanName: "Interactive Brokers",
    id: "ib",
    icon: ib,
    dateColumn: "dateTime",
    amountColumns: ["amount", "tradeMoney"],
    defaultColumns: ["currency", "description", "symbol", "quantity", "tradePrice", "ibCommission", "type"],
  },
];

export const getRealAmount = (transaction: RealTransaction, amountColumns: RealTransactionField[]) => {
  for (const column of amountColumns) {
    if (column in transaction) {
      const amount = transaction[column];
      if (amount) {
        return amount;
      }
    }
  }
  return undefined;
};
