import type { JSONContent } from "@tiptap/core";
import { Editor } from "@tiptap/core";
import MarkdownIt from "markdown-it";
import StarterKit from "@tiptap/starter-kit";

interface CodeBlock {
  start: number;
  end: number;
  language: string;
  content: string;
}

const md = MarkdownIt();

const resuableEditor = new Editor({
  extensions: [StarterKit],
  editable: false,
});

export function parseMarkdownToTipTap(content: string): JSONContent {
  const doc: JSONContent = {
    type: "doc",
    content: [],
  };

  // Find all code blocks (both complete and incomplete)
  const codeBlocks: CodeBlock[] = [];
  const codeBlockRegex = /```(\w*)\n?([\s\S]*?)(?:```|$)/g;
  let match;

  while ((match = codeBlockRegex.exec(content)) !== null) {
    codeBlocks.push({
      start: match.index,
      end: match.index + match[0].length,
      language: match[1] || "text",
      content: match[2] || "",
    });
  }

  let currentPos = 0;

  for (const block of codeBlocks) {
    // Add text before this code block
    if (block.start > currentPos) {
      const textBefore = content.slice(currentPos, block.start);
      if (textBefore.trim()) {
        addTextContent(doc, textBefore);
      }
    }

    // Add the code block
    doc.content!.push({
      type: "codeBlock",
      attrs: {
        language: block.language,
      },
      content: [
        {
          type: "text",
          text: block.content.trim() || " ",
        },
      ],
    });

    currentPos = block.end;
  }

  // Add remaining text after last code block
  if (currentPos < content.length) {
    const textAfter = content.slice(currentPos);
    if (textAfter.trim()) {
      addTextContent(doc, textAfter);
    }
  }

  return doc;
}

function addTextContent(doc: JSONContent, text: string) {
  // Split by newlines and create paragraphs
  const lines = text.split("\n");

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    const result = md.render(line);

    resuableEditor.commands.setContent(result, { emitUpdate: false });
    const jsonContent = resuableEditor.getJSON();

    if (line.trim()) {
      doc.content!.push(
        ...jsonContent.content.filter(
          (content) => content.content !== undefined,
        ),
      );
    }
  }

  // Remove empty paragraphs at the end
  while (doc.content!.length > 0) {
    const lastItem = doc.content![doc.content!.length - 1];
    if (
      lastItem.type === "paragraph" &&
      (!lastItem.content || lastItem.content.length === 0)
    ) {
      doc.content!.pop();
    } else {
      break;
    }
  }
}

export function textToHtml(text: string): string {
  resuableEditor.commands.setContent(text, { emitUpdate: false });
  return resuableEditor.getHTML();
}

// Function to detect if content contains incomplete code blocks
export function hasIncompleteCodeBlock(content: string): boolean {
  // Count opening ``` that don't have closing ```
  const openings = (content.match(/```/g) || []).length;
  return openings % 2 !== 0;
}
