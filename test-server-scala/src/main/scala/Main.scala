import io.vertx.core.AsyncResult
import io.vertx.scala.core.Vertx
import io.vertx.ext.scala.eventbus.bridge.tcp.TcpEventBusBridge
import io.vertx.lang.scala.json.Json
import io.vertx.scala.ext.bridge.{BridgeOptions, PermittedOptions}
object Main{
    def main(args: Array[String]): Unit = {
        val vertx=Vertx.vertx()
        val eb=vertx.eventBus()
        val netVertxOptions = BridgeOptions().addInboundPermitted(PermittedOptions().setAddress("test")).addOutboundPermitted(PermittedOptions().setAddress("test"))
        val bridge = TcpEventBusBridge.create(vertx, netVertxOptions)
        eb.consumer("test",(message:io.vertx.scala.core.eventbus.Message[java.lang.Object])=>{
            println("receive from test",message.body())
            message.fail(1,"test fail message")
            Thread.sleep(1000)
            message.reply(Json.emptyObj().put("ccc","dddd"))
            //message.reply(Json.emptyObj().put("ccc","ddddededede"))
        })
        bridge.listen(12345, "127.0.0.1", (res: AsyncResult[TcpEventBusBridge]) => {
            if(res.succeeded()){
                println("tcp bridge success")
            }
        })
        var l=scala.io.StdIn
    }
}