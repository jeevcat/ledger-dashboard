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
    icon: "https://play-lh.googleusercontent.com/3rYykP3ync1dDBDFRlAZy1eb4LIaV_IuG-bCVJhba_Sa6jA4gdWDxCst-EQS-SGUzQM",
    defaultColumns: ["made_on", "description", "payee", "category", "amount"],
  },
  {
    humanName: "N26",
    id: "n26",
    icon: "https://play-lh.googleusercontent.com/85SeCCkigrkJV5b7aHUQc07CozV1xLaIK_UZ7A1_VeeXK9k4CTMIWsXGSfQJrGYHGWs",
    defaultColumns: ["visibleTS", "referenceText", "partnerName", "merchantName", "mcc", "amount"],
  },
  {
    humanName: "Interactive Brokers",
    id: "ib",
    icon: "https://play-lh.googleusercontent.com/t6vYo11fzpC32nXXXVS1_Pg-bbswrF1X2f0rdHv_X2DLAJkqdWb-3FfCIUzA6MV7CTM",
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
