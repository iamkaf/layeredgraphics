import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

export default defineConfig({
  site: "https://layered.graphics",
  integrations: [
    starlight({
      title: "Layered Graphics",
      description: "Headless graphics authoring for every app.",
      customCss: ["./src/styles/global.css"],
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/iamkaf/layeredgraphics" },
      ],
      sidebar: [
        { label: "Start", items: [
          { label: "Introduction", link: "/docs/" },
          { label: "CLI quickstart", link: "/docs/cli/" },
          { label: "Browser preview", link: "/docs/browser/" },
          { label: "Live rendering proof", link: "/playground/" },
        ] },
        { label: "Concepts", items: [{ autogenerate: { directory: "docs/concepts" } }] },
        { label: "Project", items: [
          { label: "Roadmap", link: "/docs/project/roadmap/" },
          { label: "Technology stack", link: "/docs/project/technology/" },
          { label: "Runtime support", link: "/docs/project/support/" },
          { label: "Benchmarks", link: "/docs/project/benchmarks/" },
          { label: "Phase 1/2 audit", link: "/docs/project/audit/" },
        ] },
      ],
    }),
  ],
});
