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

export interface Transaction {
  tdescription: string;
  tdate: string;
  ttags: string[][];
  tpostings?: Posting[];
}

export const getAmount = (t: Transaction, account: string): number => {
  for (const p of t.tpostings ?? []) {
    if (p.paccount.includes(account)) {
      return getPostingAmount(p);
    }
  }
  return 0;
};

export const getId = (t: Transaction): string | undefined => {
  for (const tag of t.ttags) {
    if (tag[0] === "uuid") {
      return tag[1];
    }
  }
};

export const getDate = (t: Transaction): string => asDate(t.tdate);

export const getTargetAccount = (t: Transaction, sourceAccount: string): string | undefined => {
  return t.tpostings?.find((p: Posting) => !p.paccount.includes(sourceAccount))?.paccount;
};
