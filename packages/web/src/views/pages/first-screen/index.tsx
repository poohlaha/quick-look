/**
 * @fileOverview 资源未加载完成前的首屏加载
 * @date 2023-04-21
 * @author poohlaha
 */
import React from 'react'
import GlobalLoading from '@views/components/loading'

const FirstScreen = () => {
  return (
    <div className="first-screen">
      <GlobalLoading show={true} />
    </div>
  )
}

export default FirstScreen
