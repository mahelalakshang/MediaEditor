import { useCallback, useEffect, useState } from 'react';

export type Route = { name: 'library' } | { name: 'asset'; id: string };

function parse(hash: string): Route {
  const match = hash.match(/^#\/asset\/([^/]+)/);
  if (match) return { name: 'asset', id: decodeURIComponent(match[1]) };
  return { name: 'library' };
}

export function useHashRoute() {
  const [route, setRoute] = useState<Route>(() => parse(window.location.hash));

  useEffect(() => {
    const onChange = () => setRoute(parse(window.location.hash));
    window.addEventListener('hashchange', onChange);
    return () => window.removeEventListener('hashchange', onChange);
  }, []);

  const navigate = useCallback((to: Route) => {
    window.location.hash = to.name === 'asset' ? `#/asset/${encodeURIComponent(to.id)}` : '#/';
  }, []);

  return { route, navigate };
}
