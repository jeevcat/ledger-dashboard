import ib from "../Images/ib.png";
import ing from "../Images/ing.png";
import n26 from "../Images/n26.png";
import { RealTransactionField } from "./ImportRow";

export type ImportAccountType = "ing" | "n26" | "ib";

export interface ImportAccount {
  humanName: string;
  id: ImportAccountType;
  icon: string;
  defaultColumns: RealTransactionField[];
}

export const ImportAccounts: ImportAccount[] = [
  {
    humanName: "ING DiBa",
    id: "ing",
    icon: ing,
    defaultColumns: ["made_on", "description", "payee", "category", "amount"],
  },
  {
    humanName: "N26",
    id: "n26",
    icon: n26,
    defaultColumns: ["visibleTS", "referenceText", "partnerName", "merchantName", "mcc", "amount"],
  },
  {
    humanName: "Interactive Brokers",
    id: "ib",
    icon: ib,
    defaultColumns: [
      "dateTime",
      "description",
      "currency",
      "amount",
      "symbol",
      "quantity",
      "tradePrice",
      "tradeMoney",
      "ibCommission",
      "type",
    ],
  },
];
