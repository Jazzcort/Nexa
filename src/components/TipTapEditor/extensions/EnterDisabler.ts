import { Extension } from "@tiptap/core";

export const EnterDisabler = Extension.create({
  // We'll give our extension a name
  name: "enterDisabler",

  // The magic happens in the keyboard shortcuts
  addKeyboardShortcuts() {
    return {
      // When the user presses "Enter"
      Enter: () => {
        // We return `true` to tell the editor that we've handled this
        // key press and it should stop trying to do anything else.
        return true;
      },
    };
  },
});
