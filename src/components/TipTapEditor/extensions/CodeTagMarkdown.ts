import { Node, mergeAttributes } from "@tiptap/core";

export const CodeTagMarkdown = Node.create({
  name: "code",
  group: "block",
  content: "block+",
  parseHTML() {
    return [{ tag: 'code[class="inlineCodeTag"]' }];
  },
  renderHTML({ HTMLAttributes }) {
    return [
      "code",
      mergeAttributes(HTMLAttributes, { class: "inliceCodeTag" }),
      0,
    ];
  },
});
