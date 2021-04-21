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
  ptags: string[][];
}

export const getPostingAmount = (p: Posting): number => {
  // TODO: different currencies?
  return p.pamount[0].aquantity.floatingPoint;
};

export interface HledgerTransaction {
  tdescription: string;
  tdate: string;
  ttags: string[][];
  tpostings?: Posting[];
}

export const getAmount = (t: HledgerTransaction, account: string): number => {
  for (const p of t.tpostings ?? []) {
    if (p.paccount.toLowerCase().includes(account.toLowerCase())) {
      return getPostingAmount(p);
    }
  }
  return 0;
};

export const getId = (t: HledgerTransaction): string | undefined => {
  for (const tag of t.ttags) {
    if (tag[0] === "uuid") {
      return tag[1];
    }
  }
  for (const p of t.tpostings ?? []) {
    for (const tag of p.ptags) {
      if (tag[0] === "uuid") {
        return tag[1];
      }
    }
  }
};

export const getDate = (t: HledgerTransaction): string => asDate(t.tdate);

export const getMatchingAccount = (t: HledgerTransaction, importAccountId: string): string | undefined => {
  return t.tpostings?.find((p: Posting) => p.paccount.toLowerCase().includes(importAccountId.toLowerCase()))?.paccount;
};

export const getUnmatchingAccount = (t: HledgerTransaction, importAccountId: string): string | undefined => {
  return t.tpostings?.find((p: Posting) => !p.paccount.toLowerCase().includes(":" + importAccountId.toLowerCase()))
    ?.paccount;
};
