import StarterKit from "@tiptap/starter-kit";
import { FontSize, TextStyle, Color } from "@tiptap/extension-text-style";
import { SvelteNodeViewRenderer } from "svelte-tiptap";
import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
import CodeBlock from "./CodeBlock.svelte";
import Text from "@tiptap/extension-text";
import Paragraph from "@tiptap/extension-paragraph";
import Document from "@tiptap/extension-document";
import HardBreak from "@tiptap/extension-hard-break";
import { EnterDisabler } from "./extensions/EnterDisabler";
import Bold from "@tiptap/extension-bold";
import Code from "@tiptap/extension-code";
import { all, createLowlight } from "lowlight";
import js from "highlight.js/lib/languages/javascript";
import rust from "highlight.js/lib/languages/rust";
import python from "highlight.js/lib/languages/python";
import typescript from "highlight.js/lib/languages/typescript";
import java from "highlight.js/lib/languages/java";
import cpp from "highlight.js/lib/languages/cpp";
import json from "highlight.js/lib/languages/json";
import bash from "highlight.js/lib/languages/bash";

const lowlight = createLowlight(all);
lowlight.register("rust", rust);
lowlight.register("js", js);
lowlight.register("javascript", js);
lowlight.register("python", python);
lowlight.register("py", python);
lowlight.register("typescript", typescript);
lowlight.register("ts", typescript);
lowlight.register("java", java);
lowlight.register("cpp", cpp);
lowlight.register("c++", cpp);
lowlight.register("json", json);
lowlight.register("bash", bash);
lowlight.register("sh", bash);

export const AssistantEditorExtension = [
  StarterKit.configure({
    bold: false,
    codeBlock: false,
    code: false,
    trailingNode: false,
    orderedList: {
      HTMLAttributes: {
        class: "list-decimal",
      },
    },
    bulletList: {
      HTMLAttributes: {
        class: "list-disc",
      },
    },
    heading: {
      levels: [1, 2, 3, 4],
    },
    link: {
      openOnClick: false,
      autolink: true,
      linkOnPaste: true,
    },
  }),
  CodeBlockLowlight.configure({
    lowlight,
  }).extend({
    addNodeView() {
      return SvelteNodeViewRenderer(CodeBlock);
    },
  }),
  FontSize,
  TextStyle,
  Color,
  Code,
  Bold,
  EnterDisabler,
];

export const UserEditorExtension = [
  Document,
  Paragraph,
  Text,
  HardBreak,
  EnterDisabler,
];
// export const UserEditorExtension = [StarterKit];
