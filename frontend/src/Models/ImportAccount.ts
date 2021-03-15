import { RealTransactionField } from "./ImportRow";

export interface ImportAccount {
  humanName: string;
  path: string;
  icon: string;
  defaultColumns: RealTransactionField[];
}

export const ImportAccounts: ImportAccount[] = [
  {
    humanName: "N26",
    path: "n26",
    icon:
      "https://play-lh.googleusercontent.com/85SeCCkigrkJV5b7aHUQc07CozV1xLaIK_UZ7A1_VeeXK9k4CTMIWsXGSfQJrGYHGWs=s24",
    defaultColumns: ["visibleTS", "referenceText", "partnerName", "merchantName", "amount"],
  },
  {
    humanName: "Interactive Brokers",
    path: "ib",
    icon:
      "https://play-lh.googleusercontent.com/t6vYo11fzpC32nXXXVS1_Pg-bbswrF1X2f0rdHv_X2DLAJkqdWb-3FfCIUzA6MV7CTM=s24",
    defaultColumns: ["Date", "Description", "Currency", "Amount"],
  },
];
