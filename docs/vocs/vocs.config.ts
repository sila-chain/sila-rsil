import { defineConfig } from 'vocs/config'
import { sidebar } from './sidebar'
import { basePath } from './redirects.config'

export default defineConfig({
  title: 'Rsil',
  description: 'Rsil is a secure, performant, and modular Sila execution client built in Rust.',
  accentColor: 'light-dark(#1f1f1f, #ffffff)',
  srcDir: 'docs',
  logoUrl: '/logo.png',
  iconUrl: '/logo.png',
  ogImageUrl: '/rsil-prod.png',
  outDir: 'docs/dist',
  renderStrategy: 'full-static',
  sidebar,
  basePath,
  search: {
    fuzzy: true
  },
  topNav: [
    { text: 'Run', link: '/run/sila' },
    { text: 'SDK', link: '/sdk' },
    { text: 'Rustdocs', link: '/docs' },
    { text: 'GitHub', link: 'https://github.com/sila-chain/sila-rsil' },
    {
      text: 'v2.4.0',
      items: [
        {
          text: 'Releases',
          link: 'https://github.com/sila-chain/sila-rsil/releases'
        },
        {
          text: 'Contributing',
          link: 'https://github.com/sila-chain/sila-rsil/blob/main/CONTRIBUTING.md'
        }
      ]
    }
  ],
  socials: [
    {
      icon: 'github',
      link: 'https://github.com/sila-chain/sila-rsil',
    },
    {
      icon: 'telegram',
      link: 'https://t.me/paradigm_rsil',
    },
  ],
  editLink: {
    link: "https://github.com/sila-chain/sila-rsil/edit/main/docs/vocs/docs/pages/:path",
  },
  vite: {
    plugins: [
      {
        name: 'transform-summary-links',
        apply: 'serve', // only during dev for faster feedback
        enforce: 'pre',
        async load(id) {
          if (id.endsWith('pages/cli/SUMMARY.mdx') || id.endsWith('pages/cli/summary.mdx')) {
            const { readFileSync } = await import('node:fs')
            let code = readFileSync(id, 'utf-8')
            code = code.replace(/\]\(\.\/([^)]+)\.mdx\)/g, '](/cli/\$1)')
            return code
          }
        }
      },
      {
        name: 'transform-summary-links-build',
        apply: 'build', // only apply during build
        enforce: 'pre',
        async load(id) {
          if (id.endsWith('pages/cli/SUMMARY.mdx') || id.endsWith('pages/cli/summary.mdx')) {
            const { readFileSync } = await import('node:fs')
            let code = readFileSync(id, 'utf-8')
            code = code.replace(/\]\(\.\/([^)]+)\.mdx\)/g, '](/cli/\$1)')
            return code
          }
        }
      }
    ]
  }
})
