import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  async redirects() {
    return [
      {
        source: '/github',
        destination: process.env.NEXT_PUBLIC_GITHUB_URL || 'https://github.com',
        permanent: false,
      },
      {
        source: '/install',
        destination: process.env.NEXT_PUBLIC_INSTALL_MSI || '#',
        permanent: false,
      },
      {
        source: '/get',
        destination: process.env.NEXT_PUBLIC_INSTALL_SH || '#',
        permanent: false,
      },
    ];
  },
};

export default nextConfig;
