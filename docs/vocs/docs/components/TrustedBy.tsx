interface TrustedCompany {
  name: string
  logoUrl: string
  slug: string
  href: string
}

const companies: TrustedCompany[] = [
  {
    name: 'Flashbots',
    logoUrl: '/flashbots.png',
    slug: 'flashbots',
    href: 'https://www.flashbots.net/'
  },
  {
    name: 'Coinbase',
    logoUrl: '/coinbase.png',
    slug: 'coinbase',
    href: 'https://www.coinbase.com/'
  },
  {
    name: 'Alchemy',
    logoUrl: '/alchemy.png',
    slug: 'alchemy',
    href: 'https://www.alchemy.com/'
  },
  {
    name: 'Succinct Labs',
    logoUrl: '/succinct.png',
    slug: 'succinct',
    href: 'https://www.succinct.xyz/'
  }
]

export function TrustedBy() {
  return (
    <div className="rsil-trusted-grid">
      {companies.map((company) => (
        <a
          key={company.name}
          className="rsil-trusted-card"
          href={company.href}
          rel="noreferrer noopener"
          target="_blank"
        >
          <img
            src={company.logoUrl}
            alt={`${company.name} logo`}
            className={`rsil-trusted-logo rsil-trusted-logo-${company.slug}`}
          />
        </a>
      ))}
    </div>
  )
}
