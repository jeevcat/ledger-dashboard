import { asDate } from "../Utils/TextUtils";

export interface Amount {
  acommodity: string;
  aquantity: {
    floatingPoint: number;
    decimalPlaces: number;
    decimalMantissa: number;
  };
}

export interface Posting {
  paccount: string;
  pamount: Amount[];
}

export const getPostingAmount = (p: Posting): number => {
  // TODO: different currencies?
  return p.pamount[0].aquantity.floatingPoint;
};

export interface RecordedTransaction {
  tdescription: string;
  tdate: string;
  ttags: string[][];
  tpostings?: Posting[];
}

export const getAmount = (t: RecordedTransaction, account: string): number => {
  for (const p of t.tpostings ?? []) {
    if (p.paccount.toLowerCase().includes(account.toLowerCase())) {
      return getPostingAmount(p);
    }
  }
  return 0;
};

export const getId = (t: RecordedTransaction): string | undefined => {
  for (const tag of t.ttags) {
    if (tag[0] === "uuid") {
      return tag[1];
    }
  }
};

export const getDate = (t: RecordedTransaction): string => asDate(t.tdate);

export const getTargetAccount = (t: RecordedTransaction, importAccountId: string): string | undefined => {
  return t.tpostings?.find((p: Posting) => !p.paccount.toLowerCase().includes(":" + importAccountId.toLowerCase()))
    ?.paccount;
};
