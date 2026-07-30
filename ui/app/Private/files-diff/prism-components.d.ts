/// `@types/prismjs` declares the core only. Each component file is imported
/// purely for its side effect — it registers one grammar on the global `Prism` —
/// so these modules need a name tsc accepts, not a shape.
declare module "prismjs/components/*" {}
