export function extractGdscriptFences(markdown) {
  const fences = [];
  const fence = /```gdscript\r?\n([\s\S]*?)```/g;
  let match;
  while ((match = fence.exec(markdown)) !== null) {
    fences.push(match[1]);
  }
  return fences;
}
