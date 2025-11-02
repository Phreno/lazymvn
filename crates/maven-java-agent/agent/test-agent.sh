#!/bin/bash
# Test manual du Java Agent Log4j Reconfiguration

set -e

echo "=== Test Java Agent Log4j Reconfiguration ==="
echo

# 1. Créer un fichier Log4j de test
TEST_LOG4J="/tmp/test-log4j.properties"
cat > "$TEST_LOG4J" <<EOF
log4j.rootLogger=DEBUG, CONSOLE
log4j.appender.CONSOLE=org.apache.log4j.ConsoleAppender
log4j.appender.CONSOLE.layout=org.apache.log4j.PatternLayout
log4j.appender.CONSOLE.layout.ConversionPattern=[%p][%c] %m%n
EOF

echo "✓ Fichier Log4j de test créé: $TEST_LOG4J"
cat "$TEST_LOG4J"
echo

# 2. Créer une application de test simple
TEST_APP="/tmp/TestApp.java"
cat > "$TEST_APP" <<'EOF'
public class TestApp {
    public static void main(String[] args) throws Exception {
        System.out.println("[TestApp] Starting application...");
        
        // Simuler le délai de chargement de l'application
        Thread.sleep(1000);
        
        // Essayer d'utiliser Log4j si disponible
        try {
            Class<?> loggerClass = Class.forName("org.apache.log4j.Logger");
            Object logger = loggerClass.getMethod("getLogger", Class.class).invoke(null, TestApp.class);
            loggerClass.getMethod("info", Object.class).invoke(logger, "This is a test log message");
            System.out.println("[TestApp] Log4j logger used successfully");
        } catch (ClassNotFoundException e) {
            System.out.println("[TestApp] Log4j not available (expected in this test)");
        }
        
        System.out.println("[TestApp] Application finished");
    }
}
EOF

echo "✓ Application de test créée: $TEST_APP"
echo

# 3. Compiler l'application de test
javac "$TEST_APP" -d /tmp
echo "✓ Application compilée"
echo

# 4. Exécuter avec l'agent
AGENT_JAR="target/log4j-reconfig-agent-0.1.0.jar"
echo "=== Exécution avec Java Agent ==="
echo "Command: java -javaagent:$AGENT_JAR -Dlog4j.configuration=file://$TEST_LOG4J -cp /tmp TestApp"
echo

java -javaagent:"$AGENT_JAR" -Dlog4j.configuration="file://$TEST_LOG4J" -cp /tmp TestApp

echo
echo "=== Test terminé ==="
echo
echo "Attendu:"
echo "  - '[LazyMVN Agent] Starting Log4j reconfiguration agent...'"
echo "  - '[LazyMVN Agent] Will reconfigure Log4j...'"
echo "  - '[TestApp] Starting application...'"
echo "  - '[TestApp] Log4j not available (expected in this test)'"
echo "  - '[LazyMVN Agent] Log4j classes not found...'"
echo "  - '[TestApp] Application finished'"
