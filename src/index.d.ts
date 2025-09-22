declare module "turndown-plugin-gfm" {
  export function gfm(turndownService: any): void;
  export function highlightedCodeBlock(turndownService: any): void;
  export function strikethrough(turndownService: any): void;
  export function tables(turndownService: any): void;
  export function taskListItems(turndownService: any): void;
}
