export interface Balances {
  balances: Balance[];
}

export interface Balance {
  commodity: string;
  real: number;
  realEuro?: number;
  hledger: number;
}
