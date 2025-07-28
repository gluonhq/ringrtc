package io.privacyresearch.tring;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.logging.Level;
import java.util.logging.Logger;

public class TringExecutorService {

    private static final Logger LOG = Logger.getLogger(TringExecutorService.class.getName());

    private final ExecutorService executor = Executors.newFixedThreadPool(1);

    public void executeRequest(Runnable runnable) {
        LOG.info("Executing request "+runnable);
        Future<?> submit = executor.submit(runnable);
        LOG.info("Execution state = " + submit.state());
    }
}
