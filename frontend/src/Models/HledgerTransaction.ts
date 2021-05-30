import { asDate, asEuro } from "../Utils/TextUtils";

export interface Amount {
  acommodity: string;
  aquantity: {
    floatingPoint: number;
    decimalPlaces: number;
    decimalMantissa: number;
  };
  aprice?: {
    tag: string;
    contents: Amount;
  };
}

export interface Posting {
  paccount: string;
  pamount: Amount[];
  ptags: string[][];
  pcomment: string;
}

export interface FormattedAmount {
  formatted: string;
  positive: boolean;
}

export const getPostingAmount = (p: Posting, negative?: boolean): FormattedAmount => {
  /* TODO: different currencies?*/
  if (p.pamount[0].aprice) {
    const value = p.pamount[0].aprice.contents.aquantity.floatingPoint;
    const sym = p.pamount[0].acommodity;
    const sym_amount = p.pamount[0].aquantity.floatingPoint;
    const cost = asEuro(negative ? -value : value);
    const separator = p.pamount[0].aprice.tag === "TotalPrice" ? "@@" : "@";

    return {
      formatted: `${sym_amount} ${sym} ${separator} ${cost}`,
      positive: value > 0,
    };
  }
  const value = p.pamount[0].aquantity.floatingPoint;
  const cost = asEuro(negative ? -value : value);
  return {
    formatted: cost,
    positive: value > 0,
  };
};

export interface HledgerTransaction {
  tdescription: string;
  tdate: string;
  ttags?: string[][];
  tpostings?: Posting[];
}

export const getAmount = (t: HledgerTransaction, account: string): FormattedAmount | undefined => {
  for (const p of t.tpostings ?? []) {
    if (p.paccount.toLowerCase().includes(account.toLowerCase())) {
      return getPostingAmount(p);
    }
  }
};

export const getId = (t: HledgerTransaction): string | undefined => {
  if (t.ttags) {
    for (const tag of t.ttags) {
      if (tag[0] === "uuid") {
        return tag[1];
      }
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

export const getAccounts = (t: HledgerTransaction): string[] => (t.tpostings ?? []).map((p: Posting) => p.paccount);

export const getMatchingAccount = (t: HledgerTransaction, importAccountId: string): string | undefined => {
  return t.tpostings?.find((p: Posting) => p.paccount.toLowerCase().includes(importAccountId.toLowerCase()))?.paccount;
};

export const getUnmatchingAccounts = (t: HledgerTransaction, importAccountId: string): string[] =>
  (t.tpostings ?? [])
    .filter((p: Posting) => !p.paccount.toLowerCase().includes(":" + importAccountId.toLowerCase()))
    .map((p: Posting) => p.paccount);
