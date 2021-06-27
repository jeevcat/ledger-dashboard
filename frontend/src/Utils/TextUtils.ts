export const toTitleCase = (text: string) =>
  text
    // insert a space between lower & upper
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    // space before last upper in a sequence followed by lower
    .replace(/\b([A-Z]+)([A-Z])([a-z])/, "$1 $2$3")
    // uppercase the first character
    .replace(/^./, function (str) {
      return str.toUpperCase();
    });

export function keys<O>(o: O) {
  return Object.keys(o) as (keyof O)[];
}

export function asEuro(amount: number, cents: boolean = true, precision: number = 2): string {
  return asCurrency(amount, "EUR", cents, precision);
}

export function asCurrency(amount: number, currency: string, cents: boolean = true, precision: number = 2): string {
  const formatter = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: currency,
    maximumFractionDigits: cents ? precision : 0,
    minimumFractionDigits: cents ? precision : 0,
  });
  return formatter.format(amount);
}

export const asDate = (dateString: string) => {
  const date: Date = new Date(dateString);
  return date.toLocaleDateString("en-DE");
};
