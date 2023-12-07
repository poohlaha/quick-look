import React from 'react'
import RouterUrls from '@route/router.url.toml'

/**
 * 页面找不到
 */
const PageNotFound: React.FC<IRouterProps> = (props: IRouterProps) => {
  return (
    <div className="not-found">
      <span>页面未找到.</span>
      <div className="home-button" onClick={() => (window.location.href = RouterUrls.MAIN_URL)}>
        返回首页
      </div>
    </div>
  )
}

export default PageNotFound
